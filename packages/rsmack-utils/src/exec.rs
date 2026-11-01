//! Reusable proc-macro **entrypoint drivers**.
//!
//! Every rsmack-based macro repeats the same dance: parse the attribute meta with
//! [darling], parse the annotated item with [syn], call the implementation, then flush
//! the collected diagnostics. [`attr_entrypoint`], [`func_entrypoint`] and
//! [`derive_entrypoint`] implement that dance once (DRY) and route *every* failure
//! through [`Logr`](crate::logr::Logr), so users always see the same
//! `#[<crate>::<macro>]` prefixed diagnostics.
//!
//! Generated macros call them directly:
//!
//! ```rust,ignore
//! #[proc_macro_attribute]
//! pub fn my_macro(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
//!     rsmack_utils::exec::attr_entrypoint(
//!         attr.into(),
//!         item.into(),
//!         concat!(std::module_path!(), "::my_macro"),
//!         |args, item, env| crate::impls::my_macro::exec(args, item, env),
//!     )
//!     .into()
//! }
//! ```
//!
//! [darling]: https://docs.rs/darling
//! [syn]: https://docs.rs/syn

use darling::Error as DarlingError;
use darling::ast::NestedMeta;
use proc_macro2::TokenStream;

use crate::megamac::ExecEnv;

/// Parse the macro arguments with darling.
///
/// An empty token stream (e.g. `#[my_macro]` used without parentheses) maps to
/// darling's [`FromMeta::from_word`], which unit-struct `Args` support natively.
/// Every failure is rendered as prefixed `compile_error!` tokens.
/// Parse the macro arguments with darling.
///
/// An empty token stream (e.g. `#[my_macro]` used without parentheses) parses as an
/// empty meta list: darling then takes every field default, so `Args` structs used bare
/// must be field structs (an empty one for no parameters) instead of unit structs.
/// Every failure is rendered as prefixed `compile_error!` tokens.
fn parse_args<A: darling::FromMeta>(meta: TokenStream, env: &ExecEnv) -> Result<A, TokenStream> {
    A::from_list(&parse_meta_list(meta, env)?)
        .map_err(|e| env.logr.error(e.span(), e).to_compile_error())
}

/// Parse the meta tokens into a list: an empty stream becomes an empty list.
fn parse_meta_list(meta: TokenStream, env: &ExecEnv) -> Result<Vec<NestedMeta>, TokenStream> {
    if meta.is_empty() {
        return Ok(Vec::new());
    }
    NestedMeta::parse_meta_list(meta)
        .map_err(DarlingError::from)
        .map_err(|e| env.logr.error(e.span(), e).to_compile_error())
}

/// Parse the annotated item, rendering parse failures as prefixed `compile_error!` tokens.
fn parse_item<I: syn::parse::Parse>(item: TokenStream, env: &ExecEnv) -> Result<I, TokenStream> {
    syn::parse2::<I>(item).map_err(|e| env.logr.error(e.span(), e).to_compile_error())
}

/// Entrypoint driver for `#[proc_macro_attribute]` implementations.
///
/// * `meta` — the attribute payload tokens (empty for `#[my_macro]` without parens).
/// * `item` — the annotated item tokens.
/// * `prefix` — diagnostic prefix, usually `concat!(module_path!(), "::", "my_macro")`.
/// * `exec` — the implementation, receiving parsed args, the parsed item and the env.
pub fn attr_entrypoint<A, I, F>(
    meta: TokenStream,
    item: TokenStream,
    prefix: &str,
    exec: F,
) -> TokenStream
where
    A: darling::FromMeta,
    I: syn::parse::Parse,
    F: FnOnce(A, I, &ExecEnv) -> TokenStream,
{
    let env = ExecEnv::builder(prefix).build();
    let parsed_args = match parse_args::<A>(meta, &env) {
        Ok(args) => args,
        Err(error) => return env.finish(error),
    };
    let parsed_item = match parse_item::<I>(item, &env) {
        Ok(item) => item,
        Err(error) => return env.finish(error),
    };
    env.finish(exec(parsed_args, parsed_item, &env))
}

/// Entrypoint driver for `#[proc_macro]` implementations.
///
/// See [`attr_entrypoint`] for the argument documentation.
pub fn func_entrypoint<A, F>(meta: TokenStream, prefix: &str, exec: F) -> TokenStream
where
    A: darling::FromMeta,
    F: FnOnce(A, &ExecEnv) -> TokenStream,
{
    let env = ExecEnv::builder(prefix).build();
    let parsed_args = match parse_args::<A>(meta, &env) {
        Ok(args) => args,
        Err(error) => return env.finish(error),
    };
    env.finish(exec(parsed_args, &env))
}

/// Entrypoint driver for `#[proc_macro_derive]` implementations.
///
/// See [`attr_entrypoint`] for the argument documentation.
pub fn derive_entrypoint<I, F>(item: TokenStream, prefix: &str, exec: F) -> TokenStream
where
    I: syn::parse::Parse,
    F: FnOnce(I, &ExecEnv) -> TokenStream,
{
    let env = ExecEnv::builder(prefix).build();
    let parsed_item = match parse_item::<I>(item, &env) {
        Ok(item) => item,
        Err(error) => return env.finish(error),
    };
    env.finish(exec(parsed_item, &env))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[derive(Debug, darling::FromMeta)]
    struct DemoArgs {
        /// Who to greet
        who: String,
    }

    /// Unit-style args: works bare (empty meta list) **and** with a payload via its
    /// optional flag, so every branch of the generic drivers is exercisable per
    /// instantiation.
    #[derive(Debug, Default, darling::FromMeta)]
    #[darling(default)]
    struct UnitArgs {
        /// Optional flag
        #[darling(default)]
        flag: bool,
    }

    fn env_prefix() -> &'static str {
        "test_crate::demo"
    }

    /// Every closure instantiation flows through success **and** both failure classes
    /// (args parse, item parse): the generic drivers are covered per instantiation.
    #[test]
    fn attr_entrypoint_driver() {
        let exec = |args: DemoArgs, item: syn::ItemStruct, env: &ExecEnv| {
            assert_eq!(args.who, "world");
            assert_eq!(env.prefix, env_prefix());
            quote! { #item }
        };
        // Happy path
        let output = attr_entrypoint(
            quote! { who = "world" },
            quote! { struct Demo { a: u8 } },
            env_prefix(),
            exec,
        );
        let rendered = output.to_string();
        assert!(rendered.contains("struct Demo"));
        assert!(!rendered.contains("compile_error"));
        // Missing required field -> prefixed darling error
        let output = attr_entrypoint(
            quote! { other = 1 },
            quote! { struct Demo; },
            env_prefix(),
            exec,
        );
        let rendered = output.to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[test_crate::demo]"));
        assert!(rendered.contains("who"));
        // Malformed meta (`who =`) -> prefixed darling error
        let output = attr_entrypoint(
            quote! { who = },
            quote! { struct Demo; },
            env_prefix(),
            exec,
        );
        let rendered = output.to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[test_crate::demo]"));
        // Unparsable meta (`@`) -> prefixed parse error
        let output = attr_entrypoint(quote! { @ }, quote! { struct Demo; }, env_prefix(), exec);
        let rendered = output.to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[test_crate::demo]"));
        // Item parse error -> prefixed parse error
        let output = attr_entrypoint(
            quote! { who = "x" },
            quote! { struct Demo { a: !!! } },
            env_prefix(),
            exec,
        );
        let rendered = output.to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[test_crate::demo]"));
        // Empty meta with required fields -> missing-field error
        let output = attr_entrypoint(quote! {}, quote! { struct Demo; }, env_prefix(), exec);
        assert!(output.to_string().contains("compile_error"));
    }

    #[test]
    fn attr_entrypoint_unit_args_driver() {
        let exec = |args: UnitArgs, item: syn::ItemStruct, _env: &ExecEnv| {
            let _ = args.flag;
            quote! { #item }
        };
        // Bare usage: the empty meta list takes the field defaults.
        let output = attr_entrypoint(quote! {}, quote! { struct Demo; }, env_prefix(), exec);
        let rendered = output.to_string();
        assert!(rendered.contains("struct Demo"));
        assert!(!rendered.contains("compile_error"));
        // Payload usage: `from_list` runs for this instantiation too.
        let output = attr_entrypoint(
            quote! { flag = true },
            quote! { struct Flagged; },
            env_prefix(),
            exec,
        );
        assert!(output.to_string().contains("struct Flagged"));
        // Bad payload: the parse_args error arm.
        let output = attr_entrypoint(
            quote! { flag = "not a bool" },
            quote! { struct Demo; },
            env_prefix(),
            exec,
        );
        assert!(output.to_string().contains("compile_error"));
        // Broken item: the parse_item error arm.
        let output = attr_entrypoint(
            quote! { flag = true },
            quote! { struct Demo { a: !!! } },
            env_prefix(),
            exec,
        );
        assert!(output.to_string().contains("compile_error"));
    }

    #[test]
    fn func_entrypoint_driver() {
        let exec = |_args: DemoArgs, _env: &ExecEnv| quote! { const GREETING: &str = "hi"; };
        // Happy path
        let output = func_entrypoint(quote! { who = "func" }, env_prefix(), exec);
        assert!(output.to_string().contains("GREETING"));
        assert!(!output.to_string().contains("compile_error"));
        // Unknown field -> prefixed darling error
        let output = func_entrypoint(quote! { unknown = 1 }, env_prefix(), exec);
        assert!(output.to_string().contains("compile_error"));
        // Unparsable meta
        let output = func_entrypoint(quote! { @ }, env_prefix(), exec);
        assert!(output.to_string().contains("compile_error"));
        // Empty meta with required fields
        let output = func_entrypoint(quote! {}, env_prefix(), exec);
        assert!(output.to_string().contains("compile_error"));
    }

    #[test]
    fn func_entrypoint_unit_args_driver() {
        let exec = |args: UnitArgs, _env: &ExecEnv| {
            let _ = args.flag;
            quote! { const OK: bool = true; }
        };
        // Bare usage
        let output = func_entrypoint(quote! {}, env_prefix(), exec);
        assert!(output.to_string().contains("const OK"));
        assert!(!output.to_string().contains("compile_error"));
        // Payload usage
        let output = func_entrypoint(quote! { flag = true }, env_prefix(), exec);
        assert!(output.to_string().contains("const OK"));
        // Bad payload: the parse_args error arm
        let output = func_entrypoint(quote! { flag = "not a bool" }, env_prefix(), exec);
        assert!(output.to_string().contains("compile_error"));
    }

    #[test]
    fn derive_entrypoint_driver() {
        let exec = |item: syn::DeriveInput, env: &ExecEnv| {
            assert_eq!(env.prefix, env_prefix());
            let ident = item.ident;
            quote! { impl Demo for #ident {} }
        };
        // Happy path
        let output = derive_entrypoint(quote! { struct User; }, env_prefix(), exec);
        assert!(output.to_string().contains("impl Demo for User"));
        assert!(!output.to_string().contains("compile_error"));
        // Item parse error
        let output = derive_entrypoint(quote! { struct }, env_prefix(), exec);
        let rendered = output.to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[test_crate::demo]"));
    }
}
