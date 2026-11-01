//! Procedural macro for generating other procedural macros with documentation and error
//! handling.
//!
//! This module provides the `megamac` procedural macro that can generate different types of
//! procedural macros (function-like, attribute, derive) with automatic documentation
//! generation and proper error handling.

use bon::Builder;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rsmack_utils::megamac::ExecEnv;
use std::path::{Path, PathBuf};
use strum::Display;
use syn::spanned::Spanned;
use syn::{Expr, ExprLit, Ident, Item, ItemStruct, Lit, Meta, MetaNameValue, Type};

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

/// Source of the generated macro documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Default)]
enum DocSource {
    /// `#[doc = include_str!("../README.md")]`: the calling crate must ship a README.
    Readme,
    /// Only the generated `# Parameters` docs, no README dependency.
    #[default]
    Inline,
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

impl DocSource {
    /// Converts an [`Ident`] to a [`DocSource`].
    ///
    /// # Errors
    /// Returns an error if the identifier doesn't match `readme` or `inline`.
    fn from_ident(value: &Ident) -> syn::Result<Self> {
        match value.to_string().as_str() {
            "readme" => Ok(Self::Readme),
            "inline" => Ok(Self::Inline),
            _ => Err(syn::Error::new(
                value.span(),
                format!("Unsupported doc source `{value}`, expected `readme` or `inline`"),
            )),
        }
    }
}

/// Arguments for configuring the megamac macro generation.
#[derive(Debug, FromMeta)]
pub struct Args {
    /// The type of procedural macro to generate as [`Ident`]
    pub(crate) kind: Ident,

    /// Name of the macro to generate as [`Ident`]
    pub(crate) name: Ident,

    /// Ident of the implementations module, `impls` by default
    #[darling(default)]
    pub(crate) impls: Option<Ident>,

    /// Ident of the args struct, `Args` by default
    #[darling(default, rename = "args")]
    pub(crate) arguments: Option<Ident>,

    /// Documentation source of the generated macro, `inline` by default
    #[darling(default)]
    pub(crate) doc: Option<Ident>,
}

impl Args {
    /// Ident of the args struct, defaulting to `Args`.
    fn args_ident(&self) -> Ident {
        self.arguments
            .clone()
            .unwrap_or_else(|| Ident::new("Args", self.name.span()))
    }

    /// Ident of the implementations module, defaulting to `impls`.
    fn impls_ident(&self) -> Ident {
        self.impls
            .clone()
            .unwrap_or_else(|| Ident::new("impls", self.name.span()))
    }

    /// Resolved documentation source, defaulting to [`DocSource::Inline`].
    ///
    /// # Errors
    /// Returns an error for unknown `doc` values.
    fn doc_source(&self) -> syn::Result<DocSource> {
        self.doc
            .as_ref()
            .map(DocSource::from_ident)
            .transpose()
            .map(Option::unwrap_or_default)
    }
}

pub(crate) fn run(args: &Args, env: &ExecEnv, src_folder: &Path) -> syn::Result<TokenStream> {
    let kind = MacroKind::from_ident(&args.kind).map_err(|e| env.logr.error(e.span(), e))?;
    let doc_source = args.doc_source().map_err(|e| env.logr.error(e.span(), e))?;
    let impls_ident = args.impls_ident();
    let name = &args.name;
    let macro_impl_file_ast = macro_impl_file_ast(args, env, src_folder, &impls_ident)?;
    let fields_doc = args_fields_doc(&macro_impl_file_ast, env, &args.args_ident(), &impls_ident)?;

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
    let readme_doc = match doc_source {
        DocSource::Readme => quote! { #[doc = include_str!("../README.md")] },
        DocSource::Inline => quote! {},
    };
    let doc = quote! {
        #readme_doc
        #[doc = concat!(#name_str, " procedural macro (", #kind_str, ").")]
        #[doc = "# Parameters"]
        #(#formatted_fields_doc)*
    };

    // The implementation entrypoint routes through the shared rsmack-utils drivers:
    // darling meta parsing, item parsing and diagnostics flushing are single-sourced.
    let exec_path = quote! { crate::#impls_ident::#name::exec };
    let macro_impl = match kind {
        MacroKind::Derive => {
            let derive_name = Ident::new(&stringcase::pascal_case(&name_str), name.span());
            quote! {
                #[proc_macro_derive(#derive_name, attributes(#name))]
                pub fn #name(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    rsmack_utils::exec::derive_entrypoint(
                        item.into(),
                        concat!(std::module_path!(), "::", #name_str),
                        |__item, __env| #exec_path(__item, __env),
                    )
                    .into()
                }
            }
        }
        MacroKind::Func => quote! {
            #[proc_macro]
            pub fn #name(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
                rsmack_utils::exec::func_entrypoint(
                    args.into(),
                    concat!(std::module_path!(), "::", #name_str),
                    |__args, __env| #exec_path(__args, __env),
                )
                .into()
            }
        },
        MacroKind::Attr => {
            // The parsed item type is inferred from the `exec` signature of the
            // implementation: no receiver configuration is needed anymore.
            quote! {
                #[proc_macro_attribute]
                pub fn #name(
                    attr: proc_macro::TokenStream,
                    item: proc_macro::TokenStream,
                ) -> proc_macro::TokenStream {
                    rsmack_utils::exec::attr_entrypoint(
                        attr.into(),
                        item.into(),
                        concat!(std::module_path!(), "::", #name_str),
                        |__args, __item, __env| #exec_path(__args, __item, __env),
                    )
                    .into()
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
fn macro_impl_file_ast(
    args: &Args,
    env: &ExecEnv,
    src_folder: &Path,
    impls_ident: &Ident,
) -> syn::Result<syn::File> {
    let macro_impl_file_path = impl_file_path(src_folder, impls_ident, &args.name);
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

/// Path of the implementation module of `name`: `<src_folder>/<impls>/<name>.rs`.
#[must_use]
fn impl_file_path(src_folder: &Path, impls_ident: &Ident, name: &Ident) -> PathBuf {
    src_folder
        .join(impls_ident.to_string())
        .join(format!("{name}.rs"))
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
/// Documentation problems of the fields are **collected** (non-fatal), so several
/// documentation issues are reported in one shot instead of failing on the first.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] if the args struct cannot be found.
fn args_fields_doc(
    macro_impl_file_ast: &syn::File,
    env: &ExecEnv,
    args_ident: &Ident,
    impls_ident: &Ident,
) -> syn::Result<Vec<FieldDoc>> {
    let Some(args_struct) = macro_impl_file_ast
        .items
        .iter()
        .find_map(|item| match item {
            Item::Struct(ItemStruct { ident, fields, .. }) if *ident == *args_ident => Some(fields),
            _ => None,
        })
    else {
        return Err(env.logr.call_site_error(format!(
            "Failed to find `{args_ident}` struct in `{impls_ident}`"
        )));
    };

    let fields_named = match args_struct {
        syn::Fields::Named(fields_named) => fields_named,
        // A unit `Args` struct is legitimate: the macro takes no parameters.
        syn::Fields::Unit => return Ok(vec![]),
        syn::Fields::Unnamed(_) => {
            env.logr.emit_error(
                args_struct.span(),
                format!(
                    "Unsupported `{args_ident}` struct: only named fields are documented, received `{}`",
                    args_struct.to_token_stream()
                ),
            );
            return Ok(vec![]);
        }
    };

    Ok(fields_named
        .named
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
        ExecEnv::builder("rsmack_megamac::megamac").build()
    }

    /// Local exec helper mirroring the lib entrypoint (`exec` lives in `lib.rs` so the
    /// playground bin can include this module without dead code).
    fn exec(args: &Args, env: &ExecEnv) -> TokenStream {
        let src_folder = rsmack_utils::fs::package_src_folder();
        env.finish_result(run(args, env, &src_folder))
    }

    /// Run `exec` with `CARGO_MANIFEST_DIR` pointing at this crate (guard + env + exec
    /// are serialized so parallel tests cannot race on the environment).
    fn exec_default(args: &Args) -> String {
        let _guard = with_manifest_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        exec(args, &test_env()).to_string()
    }

    /// Cover the exec error paths with controlled `impls` files: broken rust source,
    /// missing `Args` struct, tuple `Args` struct, and a mix of documented/undocumented
    /// fields.
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
            root.join("src").join("impls").join("tupleargs.rs"),
            "pub struct Args(pub String);",
        )
        .expect("write tuple args impl file");
        std::fs::write(
            root.join("src").join("impls").join("unitargs.rs"),
            "pub struct Args;",
        )
        .expect("write unit args impl file");
        std::fs::write(
            root.join("src").join("impls").join("mixed.rs"),
            r"
            use bon::Builder;
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
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to parse macro_impl_src"));

        // Valid rust file without any `Args` struct
        let args = args_from(quote! { kind = "Func", name = "noargs" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to find `Args` struct"));

        // Tuple `Args` struct: collected error, no panic, empty parameters section
        let args = args_from(quote! { kind = "Func", name = "tupleargs" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unsupported `Args` struct"));

        // Unit `Args` struct: legitimate, zero documented parameters
        let args = args_from(quote! { kind = "Func", name = "unitargs" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("# [proc_macro]"));
        assert!(!output.contains("compile_error"));
        assert!(!output.contains("Not documented"));

        // Documented, undocumented and non-doc attribute fields are all handled
        let args = args_from(quote! { kind = "Func", name = "mixed" });
        let output = exec(&args, &test_env()).to_string();
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
    fn doc_source_maps_every_value() {
        for (ident, expected) in [
            (syn::parse_quote!(readme), DocSource::Readme),
            (syn::parse_quote!(inline), DocSource::Inline),
        ] {
            assert_eq!(DocSource::from_ident(&ident).ok(), Some(expected));
        }
        let bad: Ident = syn::parse_quote!(nope);
        let err = DocSource::from_ident(&bad).expect_err("should fail");
        assert!(err.to_string().contains("Unsupported doc source `nope`"));
    }

    #[test]
    fn args_parse_minimal_and_full() {
        let minimal = args_from(quote! { kind = "Func", name = "demo" });
        assert_eq!(minimal.kind, "Func");
        assert_eq!(minimal.name, "demo");
        assert!(minimal.impls.is_none());
        assert!(minimal.arguments.is_none());
        assert!(minimal.doc.is_none());
        assert_eq!(minimal.impls_ident(), "impls");
        assert_eq!(minimal.args_ident(), "Args");
        assert_eq!(minimal.doc_source().expect("doc source"), DocSource::Inline);

        let full = args_from(quote! {
            kind = "Attr",
            name = "demo",
            impls = "attributes",
            args = "DemoArgs",
            doc = "readme"
        });
        assert_eq!(full.impls_ident(), "attributes");
        assert_eq!(full.args_ident(), "DemoArgs");
        assert_eq!(full.doc_source().expect("doc source"), DocSource::Readme);
    }

    #[test]
    fn args_parse_fails_without_required_fields() {
        let meta_list = NestedMeta::parse_meta_list(quote! { kind = "Func" }).expect("parse meta");
        assert!(Args::from_list(&meta_list).is_err());
    }

    #[test]
    fn impl_file_path_joins_impls_and_name() {
        let impls: Ident = syn::parse_quote!(attributes);
        let name: Ident = syn::parse_quote!(demo);
        assert_eq!(
            impl_file_path(Path::new("/crate/src"), &impls, &name),
            PathBuf::from("/crate/src/attributes/demo.rs")
        );
    }

    #[test]
    fn exec_func_generates_entrypoint_and_documented_parameters() {
        let args = args_from(quote! { kind = "Func", name = "megamac" });
        let output = exec_default(&args);
        assert!(output.contains("# [proc_macro]"));
        assert!(output.contains("pub fn megamac"));
        assert!(output.contains("func_entrypoint"));
        assert!(output.contains("crate :: impls :: megamac :: exec"));
        assert!(output.contains("# Parameters"));
        assert!(output.contains("* `kind`"));
        assert!(output.contains("* `name`"));
        // Default doc source is inline: no README dependency
        assert!(!output.contains("include_str"));
    }

    #[test]
    fn exec_readme_doc_embeds_the_readme() {
        let args = args_from(quote! { kind = "Func", name = "megamac", doc = "readme" });
        let output = exec_default(&args);
        assert!(output.contains("include_str ! (\"../README.md\")"));
    }

    #[test]
    fn exec_custom_impls_and_args_idents_are_honored() {
        let args = args_from(
            quote! { kind = "Func", name = "megamac", impls = "attributes", args = "MegamacArgs" },
        );
        let _guard = with_manifest_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        // The impl file `attributes/megamac.rs` does not exist: the error message proves
        // the custom impls ident was used for the lookup.
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("Failed to get macro_impl_src of megamac"));
        assert!(output.contains("attributes"));
    }

    #[test]
    fn exec_attr_generates_entrypoint_with_inferred_receiver() {
        let args = args_from(quote! { kind = "Attr", name = "megamac" });
        let output = exec_default(&args);
        assert!(output.contains("# [proc_macro_attribute]"));
        assert!(output.contains("attr_entrypoint"));
        assert!(output.contains("__item"));
        // The receiver type is inferred from the `exec` signature of the impl
        assert!(!output.contains("syn ::"));
    }

    #[test]
    fn exec_derive_pascal_cases_the_name() {
        let args = args_from(quote! { kind = "Derive", name = "megamac" });
        let output = exec_default(&args);
        assert!(output.contains("proc_macro_derive"));
        assert!(output.contains("Megamac"));
        assert!(output.contains("derive_entrypoint"));
    }

    #[test]
    fn exec_reports_unsupported_kind() {
        let args = args_from(quote! { kind = "Nope", name = "megamac" });
        let output = exec_default(&args);
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unsupported MacroKind"));
    }

    #[test]
    fn exec_reports_unsupported_doc_source() {
        let args = args_from(quote! { kind = "Func", name = "megamac", doc = "nope" });
        let output = exec_default(&args);
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unsupported doc source `nope`"));
    }

    #[test]
    fn exec_reports_missing_impl_file() {
        let args = args_from(quote! { kind = "Func", name = "does_not_exist" });
        let output = exec_default(&args);
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to get macro_impl_src"));
    }
}
