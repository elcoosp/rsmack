//! Procedural macro for generating other procedural macros with documentation and error
//! handling.
//!
//! This module provides the `megamac` procedural macro that can generate different types of
//! procedural macros (function-like, attribute, derive) with automatic documentation
//! generation and proper error handling.

use bon::Builder;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use strum::Display;
use syn::*;

/// Represents the kind of procedural macro to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
enum MacroKind {
    /// Function-like procedural macro (e.g., `my_macro!()`)
    Func,
    /// Attribute procedural macro (e.g., `#[my_macro]`)
    Attr,
    /// Derive procedural macro (e.g., `#[derive(MyMacro)]`)
    Derive,
}

/// Arguments for configuring the megamac macro generation.
#[derive(Debug, FromMeta)]
pub struct Args {
    /// The type of procedural macro to generate as [`Ident`]
    kind: Ident,

    /// Name of the macro to generate as [`Ident`]
    name: Ident,

    /// The receiver type for attribute macros (only used with [`MacroKind::Attr`])
    #[darling(default)]
    receiver: Option<Ident>,
}

impl MacroKind {
    /// Converts an [`Ident`] to a [`MacroKind`].
    ///
    /// # Errors
    /// Returns an error if the identifier doesn't match `Attr`, `Func` or `Derive`.
    fn from_ident(value: &Ident) -> syn::Result<Self> {
        match value.to_string().as_str() {
            "Attr" => Ok(Self::Attr),
            "Func" => Ok(Self::Func),
            "Derive" => Ok(Self::Derive),
            _ => Err(syn::Error::new(
                value.span(),
                format!("Unsupported MacroKind `{value}`, expected `Attr`, `Func` or `Derive`"),
            )),
        }
    }
}

/// Executes the megamac macro to generate a procedural macro implementation.
///
/// # Parameters
/// - `args`: Configuration arguments for the macro
/// - `env`: Execution environment containing context and utilities
///
/// # Returns
/// A [`TokenStream`] containing the generated procedural macro implementation.
pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    match run(&args, &env) {
        Ok(output) => env.finish(output),
        Err(error) => env.finish(error.to_compile_error()),
    }
}

fn run(args: &Args, env: &ExecEnv) -> syn::Result<TokenStream> {
    let kind = MacroKind::from_ident(&args.kind)?;
    let name = &args.name;
    let macro_impl_file_ast = macro_impl_file_ast(args, env)?;
    let fields_doc = args_fields_doc(&macro_impl_file_ast, env)?;

    // Format field documentation for inclusion in the generated macro docs
    let formatted_fields_doc = fields_doc
        .iter()
        .map(|fd| {
            let template_without_ty_qualified_path = format!(
                "* `{}` - {}\n  + type: [`{}`]",
                fd.ident,
                fd.doc.as_deref().unwrap_or("Not documented"),
                fd.ty.to_token_stream().to_string().replace(' ', ""),
            );

            quote! { #[doc = #template_without_ty_qualified_path]}
        })
        .collect::<Vec<_>>();

    let name_str = name.to_string();
    let kind_str = kind.to_string();

    // Generate the main documentation for the macro
    let doc = quote! {
        #[doc = include_str!("../README.md")]
        #[doc = concat!(#name_str, " procedural macro (", #kind_str, ").")]
        #[doc = "# Parameters"]
        #(#formatted_fields_doc)*
    };

    // Generate the appropriate macro implementation based on the kind
    let macro_impl = match kind {
        MacroKind::Derive => {
            let derive_name = Ident::new(&stringcase::pascal_case(&name_str), name.span());
            quote! {
                #[proc_macro_derive(#derive_name, attributes(#name))]
                pub fn #name(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    rsmack_utils::exec::call_derive_impls_with_args!(#name, item)
                }
            }
        }
        MacroKind::Func => quote! {
            #[proc_macro]
            pub fn #name(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
                rsmack_utils::exec::call_func_impls_with_args!(#name, args)
            }
        },
        MacroKind::Attr => {
            let receiver = args
                .receiver
                .clone()
                .unwrap_or_else(|| Ident::new("ItemStruct", name.span()));
            quote! {
                #[proc_macro_attribute]
                pub fn #name(
                    attr: proc_macro::TokenStream,
                    item: proc_macro::TokenStream,
                ) -> proc_macro::TokenStream {
                    rsmack_utils::exec::call_attr_impls_with_args!(#name, syn::#receiver, attr, item)
                }
            }
        }
    };

    Ok(quote! {
        #doc
        #macro_impl
    })
}

/// Parses and returns the AST of the macro implementation file.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] if the file cannot be read or parsed.
fn macro_impl_file_ast(args: &Args, env: &ExecEnv) -> syn::Result<syn::File> {
    let macro_impl_file_path = package_src_folder()
        .join(&env.implementations_mod_ident)
        .join(format!("{}.rs", args.name));
    let macro_impl_src = std::fs::read_to_string(&macro_impl_file_path).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to get macro_impl_src of {} at {}: {e}",
            args.name,
            macro_impl_file_path.display()
        ))
    })?;

    syn::parse_file(&macro_impl_src).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to parse macro_impl_src {}, this may happen for no real reason in your IDE, check that your project still builds with cargo: {e:?}",
            args.name
        ))
    })
}

/// Documentation information for a field in the macro arguments struct.
#[derive(Debug, Builder)]
struct FieldDoc {
    /// The field identifier
    ident: Ident,
    /// The field documentation string, if present
    doc: Option<String>,
    /// The field type
    ty: Type,
}

/// Extracts documentation for all fields in the macro arguments struct.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] if the arguments struct cannot be found.
fn args_fields_doc(macro_impl_file_ast: &syn::File, env: &ExecEnv) -> syn::Result<Vec<FieldDoc>> {
    let Some(args_struct) = macro_impl_file_ast
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(ItemStruct { ident, fields, .. }) if *ident == env.exec_args_ident => {
                Some(fields)
            }
            _ => None,
        })
    else {
        return Err(env.logr.call_site_error(format!(
            "Failed to find `{}` struct in `{}`",
            env.exec_args_ident, env.exec_fn_mod_ident
        )));
    };

    Ok(args_struct
        .iter()
        .map(|f| {
            let doc = f.attrs.iter().find_map(|a| match &a.meta {
                Meta::NameValue(MetaNameValue {
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        }),
                    path,
                    ..
                }) if path.is_ident("doc") => Some(lit_str.value()),
                _ => None,
            });
            let builder = FieldDoc::builder()
                .ident(f.ident.clone().expect("named field"))
                .ty(f.ty.clone());
            match doc {
                Some(doc) => builder.doc(doc).build(),
                None => builder.build(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;
    use quote::quote;

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    /// Serialize the tests mutating process-wide environment variables.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Point `CARGO_MANIFEST_DIR` at `manifest` for the duration of the guard.
    ///
    /// `package_src_folder` reads `CARGO_MANIFEST_DIR` at runtime, inside `exec`.
    fn with_manifest_dir(manifest: &std::path::Path) -> std::sync::MutexGuard<'static, ()> {
        let guard = env_guard();
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", manifest) };
        guard
    }

    fn test_env() -> ExecEnv {
        ExecEnv::builder(env!("CARGO_MANIFEST_DIR"), "impls", "Args", "megamac").build()
    }

    /// Run `exec` with `CARGO_MANIFEST_DIR` pointing at this crate (guard + env + exec
    /// are serialized so parallel tests cannot race on the environment).
    fn exec_default(args: Args) -> String {
        let _guard = with_manifest_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        exec(args, test_env()).to_string()
    }

    /// Cover the exec error paths with controlled `impls` files: broken rust source,
    /// missing `Args` struct, and a mix of documented/undocumented fields.
    #[test]
    fn exec_handles_broken_and_argsless_impl_files() {
        let root = std::env::temp_dir().join("rsmack-megamac-error-paths");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("broken.rs"),
            "this is not rust $$$",
        )
        .expect("write broken impl file");
        std::fs::write(
            root.join("src").join("impls").join("noargs.rs"),
            "pub struct Something;",
        )
        .expect("write argsless impl file");
        std::fs::write(
            root.join("src").join("impls").join("mixed.rs"),
            r"
            use darling::FromMeta;

            /// Mixed args
            #[derive(Debug, FromMeta)]
            pub struct Args {
                /// Documented field
                pub documented: String,
                #[darling(default)]
                pub undocumented: Option<String>,
            }

            pub fn exec(_args: Args, env: rsmack_utils::megamac::ExecEnv) -> proc_macro2::TokenStream {
                env.finish(quote::quote! {})
            }
            ",
        )
        .expect("write mixed impl file");

        let _guard = with_manifest_dir(&root);

        // Readable but unparsable impl file (guard held: use `exec` directly)
        let args = args_from(quote! { kind = "Func", name = "broken" });
        let output = exec(args, test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to parse macro_impl_src"));

        // Valid rust file without any `Args` struct
        let args = args_from(quote! { kind = "Func", name = "noargs" });
        let output = exec(args, test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to find `Args` struct"));

        // Documented, undocumented and non-doc attribute fields are all handled
        let args = args_from(quote! { kind = "Func", name = "mixed" });
        let output = exec(args, test_env()).to_string();
        assert!(output.contains("* `documented` - "));
        assert!(output.contains("Documented field"));
        assert!(output.contains("* `undocumented` - Not documented"));
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn from_ident_maps_every_kind() {
        for (ident, expected) in [
            (syn::parse_quote!(Attr), MacroKind::Attr),
            (syn::parse_quote!(Func), MacroKind::Func),
            (syn::parse_quote!(Derive), MacroKind::Derive),
        ] {
            assert_eq!(MacroKind::from_ident(&ident).ok(), Some(expected));
        }
    }

    #[test]
    fn from_ident_rejects_unknown_kind() {
        let bad: Ident = syn::parse_quote!(Nope);
        let err = MacroKind::from_ident(&bad).expect_err("should fail");
        assert!(err.to_string().contains("Unsupported MacroKind `Nope`"));
    }

    #[test]
    fn args_parse_minimal_and_full() {
        let minimal = args_from(quote! { kind = "Func", name = "demo" });
        assert_eq!(minimal.kind, "Func");
        assert_eq!(minimal.name, "demo");
        assert!(minimal.receiver.is_none());

        let full = args_from(quote! { kind = "Attr", name = "demo", receiver = "ItemStruct" });
        assert_eq!(full.receiver.expect("receiver set"), "ItemStruct");
    }

    #[test]
    fn args_parse_fails_without_required_fields() {
        let meta_list = NestedMeta::parse_meta_list(quote! { kind = "Func" }).expect("parse meta");
        assert!(Args::from_list(&meta_list).is_err());
    }

    #[test]
    fn exec_func_generates_entrypoint_and_documented_parameters() {
        let args = args_from(quote! { kind = "Func", name = "megamac" });
        let output = exec_default(args);
        assert!(output.contains("# [proc_macro]"));
        assert!(output.contains("pub fn megamac"));
        assert!(output.contains("call_func_impls_with_args ! (megamac , args)"));
        assert!(output.contains("# Parameters"));
        assert!(output.contains("* `kind`"));
        assert!(output.contains("* `name`"));
    }

    #[test]
    fn exec_attr_uses_receiver_type() {
        let args = args_from(quote! { kind = "Attr", name = "megamac", receiver = "ItemEnum" });
        let output = exec_default(args);
        assert!(output.contains("# [proc_macro_attribute]"));
        assert!(output.contains("syn :: ItemEnum"));
    }

    #[test]
    fn exec_attr_defaults_to_item_struct_receiver() {
        let args = args_from(quote! { kind = "Attr", name = "megamac" });
        let output = exec_default(args);
        assert!(output.contains("syn :: ItemStruct"));
    }

    #[test]
    fn exec_derive_pascal_cases_the_name() {
        let args = args_from(quote! { kind = "Derive", name = "megamac" });
        let output = exec_default(args);
        assert!(output.contains("proc_macro_derive"));
        assert!(output.contains("Megamac"));
        assert!(output.contains("call_derive_impls_with_args"));
    }

    #[test]
    fn exec_reports_unsupported_kind() {
        let args = args_from(quote! { kind = "Nope", name = "megamac" });
        let output = exec_default(args);
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unsupported MacroKind"));
    }

    #[test]
    fn exec_reports_missing_impl_file() {
        let args = args_from(quote! { kind = "Func", name = "does_not_exist" });
        let output = exec_default(args);
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to get macro_impl_src"));
    }
}
