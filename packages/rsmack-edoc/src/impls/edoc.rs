//! # Edoc Procedural Macro
//!
//! This module implements a procedural macro called `edoc` (documentation macro) that
//! generates documentation for struct fields by concatenating string literals and constants.
//!
//! ## Overview
//!
//! The `edoc` macro processes struct fields with `#[edoc]` attributes, evaluates expressions
//! containing string literals and constant references, and replaces them with standard
//! `#[doc]` attributes containing the concatenated documentation strings.
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! // In constants.rs
//! const PREFIX: &str = "MyApp";
//! const VERSION: &str = "1.0";
//!
//! // In struct definition
//! #[edoc(from = constants)]
//! struct MyStruct {
//!     #[edoc(("API version: ", VERSION))]
//!     version: String,
//!
//!     #[edoc(("Application: ", PREFIX, " - Main service"))]
//!     name: String,
//! }
//! ```
//!
//! This expands to:
//!
//! ```rust,ignore
//! struct MyStruct {
//!     #[doc = "API version: 1.0"]
//!     version: String,
//!
//!     #[doc = "Application: MyApp - Main service"]
//!     name: String,
//! }
//! ```

use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use std::{collections::HashMap, path::Path, path::PathBuf};
use syn::{spanned::Spanned, visit::Visit};

/// Macro arguments specifying where to find constants for documentation
///
/// Used as `#[edoc(from = path::to::module)]` to specify the module path
/// containing the constants referenced in edoc expressions.
#[derive(Debug, FromMeta)]
pub struct Args {
    /// The [`syn::Path`] to the module (`self` not handled) you want to pull constants from
    from: syn::Path,
}

/// Main macro execution function
///
/// Processes the struct with `edoc` attributes by:
/// 1. Parsing input and extracting `edoc` attributes
/// 2. Resolving constants from the specified module
/// 3. Evaluating tuple expressions in `edoc` attributes
/// 4. Replacing `edoc` attributes with `doc` attributes containing concatenated strings
pub fn exec(args: Args, item: syn::ItemStruct, env: ExecEnv) -> TokenStream {
    match run(args, item, &env) {
        Ok(output) => env.finish(output),
        Err(error) => env.finish(error.to_compile_error()),
    }
}

fn run(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let consts_file_path = consts_file_path(&package_src_folder(), args.from);
    let resolved_consts = resolve_consts(&consts_file_path, env)?;

    let mut evaluated_edoc_fields: Vec<String> = vec![];
    for field in &item.fields {
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("edoc")) {
            let expr = attr.parse_args::<syn::Expr>().map_err(|e| {
                env.logr
                    .error(attr.span(), format!("Failed to parse edoc expression: {e}"))
            })?;
            let evaluated_elems = match expr {
                syn::Expr::Tuple(tup) => evaluate_tuple_elems(tup.elems, &resolved_consts, env)?,
                _ => {
                    return Err(env.logr.call_site_error(
                        "Only Tuple supported, maybe you are missing a second element",
                    ));
                }
            };
            evaluated_edoc_fields.push(evaluated_elems.concat());
        }
    }

    let mut edoc_replaced_item = item;
    replace_edoc_attrs(&mut edoc_replaced_item, &evaluated_edoc_fields);
    Ok(quote! {
        #edoc_replaced_item
    })
}

/// Evaluates each element of an edoc tuple expression into a final string part
///
/// String literals are used as-is, constant identifiers are looked up in
/// `resolved_consts`. Unresolved identifiers emit a non-fatal error and are skipped.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] for unsupported tuple elements.
fn evaluate_tuple_elems(
    elems: syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
    resolved_consts: &HashMap<String, String>,
    env: &ExecEnv,
) -> syn::Result<Vec<String>> {
    let mut evaluated_elems: Vec<String> = vec![];
    for elem in elems {
        match elem {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => {
                // String literal - use its value directly
                evaluated_elems.push(lit.value());
            }
            syn::Expr::Path(syn::ExprPath {
                path: syn::Path { segments, .. },
                ..
            }) => {
                // Constant identifier (qualified paths resolve through their last segment)
                // - look up its resolved value
                let const_ident = segments.last().map(|s| &s.ident).expect("non empty path");
                match resolved_consts.get(&const_ident.to_string()) {
                    None => env.logr.emit_error(
                        const_ident.span(),
                        format!("Unresolved const ident {const_ident:?}"),
                    ),
                    Some(value) => evaluated_elems.push(value.clone()),
                }
            }
            _ => {
                return Err(env.logr.call_site_error(
                    "Unsupported tuple element, only string literal or ident of a const string",
                ));
            }
        }
    }
    Ok(evaluated_elems)
}

/// Converts a module path to a file system path relative to `src_folder`
///
/// Transforms a Rust module path (e.g., `my_module::submodule`) to a file system path
/// by converting `::` to `/` and appending `.rs` extension.
fn consts_file_path(src_folder: &Path, path: syn::Path) -> PathBuf {
    src_folder.join(format!(
        "{}.rs",
        path.to_token_stream()
            .to_string()
            .replace(' ', "")
            .replace("::", "/")
    ))
}

/// Replaces every `edoc` attribute of the struct fields with generated `#[doc]` attributes
fn replace_edoc_attrs(item: &mut syn::ItemStruct, docs: &[String]) {
    let mut doc_idx = 0;
    for field in &mut item.fields {
        for attr in &mut field.attrs {
            if attr.path().is_ident("edoc") {
                if let Some(doc_str) = docs.get(doc_idx) {
                    *attr = syn::parse_quote! { #[doc = #doc_str] };
                    doc_idx += 1;
                }
            }
        }
    }
}

/// Reads, parses and collects the constants of a source file
///
/// The file is parsed as a real Rust AST (`syn::File`), then every `const` item —
/// including those nested in modules — is collected with [`ConstCollector`].
///
/// # Supported Constant Types
/// * `&str` string slices
/// * `bool` boolean values
/// * `&[u8]` byte arrays (formatted with `Debug`)
///
/// Unsupported constant expressions emit a non-fatal error and are skipped.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] if the file cannot be read or parsed.
fn resolve_consts(consts_file_path: &Path, env: &ExecEnv) -> syn::Result<HashMap<String, String>> {
    let source = std::fs::read_to_string(consts_file_path).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to read constants file {}: {e}",
            consts_file_path.display()
        ))
    })?;
    let file = syn::parse_file(&source).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to parse constants file {}: {e:?}",
            consts_file_path.display()
        ))
    })?;
    let mut collector = ConstCollector {
        consts: HashMap::new(),
        env,
    };
    collector.visit_file(&file);
    Ok(collector.consts)
}

/// A [`Visit`]or collecting the literal constants of a file
struct ConstCollector<'a> {
    consts: HashMap<String, String>,
    env: &'a ExecEnv,
}

impl Visit<'_> for ConstCollector<'_> {
    fn visit_item_const(&mut self, item_const: &syn::ItemConst) {
        let const_name = item_const.ident.to_string();
        match &*item_const.expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit),
                ..
            }) => {
                // String constant - store its value
                self.consts.insert(const_name, lit.value());
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(lit),
                ..
            }) => {
                // Boolean constant - convert to string
                self.consts
                    .insert(const_name, lit.value().to_string());
            }
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::ByteStr(lit),
                ..
            }) => {
                // Byte string constant - format as debug string
                self.consts
                    .insert(const_name, format!("{:?}", lit.value()));
            }
            expr => self.env.logr.emit_error(
                item_const.span(),
                format!(
                    "Unexpected const item expression for `{const_name}`, expected a string, bool or byte string literal, received: `{expr:?}`"
                ),
            ),
        }
        syn::visit::visit_item_const(self, item_const);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;
    use quote::quote;
    use std::fs;

    fn env() -> ExecEnv {
        ExecEnv::builder("rsmack_edoc::impls", "impls", "Args", "edoc").build()
    }

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    fn write_consts_file(root: &Path, content: &str) {
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("src").join("consts.rs"), content).expect("write consts");
    }

    /// One sequential test mutating `CARGO_MANIFEST_DIR` (process-wide env).
    #[test]
    fn exec_end_to_end_with_real_constants_file() {
        let root = std::env::temp_dir().join("rsmack-edoc-tests");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        // SAFETY: only test in this binary mutating the environment
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
        write_consts_file(
            &root,
            r#"
            pub const APP: &str = "myapp";
            pub const VERSION: &str = "1.2.3";
            pub const VERBOSE: bool = true;
            pub const MAGIC: &[u8] = b"ab";
            pub const NOT_A_LITERAL: &str = concat!("unsupported");
            mod nested {
                pub const DEEP: &str = "deep";
            }
            pub fn not_a_const() {}
            "#,
        );

        // Happy path: literals + resolved consts, nested mod const, other attrs preserved
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            #[derive(Debug)]
            struct Client {
                /// Overwritten doc
                #[edoc(("App: ", APP, " v", VERSION))]
                name: String,
                #[edoc(("Verbose: ", VERBOSE))]
                verbose: bool,
                #[edoc(("Deep: ", nested::DEEP))]
                deep: String,
                #[edoc(("Magic: ", MAGIC))]
                magic: String,
                #[serde(rename = "n")]
                #[edoc(("Fallback ", "plain"))]
                other: u8,
                #[edoc(("Missing: ", NOPE))]
                missing: String,
                without_edoc: u8,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(
            output.contains("# [doc = \"App: myapp v1.2.3\"]"),
            "unexpected output: {output}"
        );
        assert!(output.contains("# [doc = \"Verbose: true\"]"));
        assert!(output.contains("# [doc = \"Deep: deep\"]"));
        assert!(output.contains("# [doc = \"Magic: [97, 98]\"]"));
        // non-edoc attributes are preserved
        assert!(output.contains("serde (rename"));
        assert!(output.contains("derive (Debug)"));
        // fields without edoc attribute are kept untouched
        assert!(output.contains("without_edoc : u8"));
        // unresolved const emits a collected compile_error, other fields keep their docs
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unresolved const ident"));
        assert!(output.contains("# [doc = \"Missing: \"]"));

        // Unsupported const expression in the file: collected error, but other consts still resolve
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("App: ", APP))]
                name: String,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("Unexpected const item expression for `NOT_A_LITERAL`"));
        assert!(output.contains("# [doc = \"App: myapp\"]"));

        // Non-tuple edoc expression: fatal error tokens
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc("not a tuple")]
                name: String,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("Only Tuple supported"));

        // Unsupported tuple element: fatal error tokens
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("literal ok ", 1 + 1))]
                name: String,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("Unsupported tuple element"));

        // Malformed edoc expression: fatal error tokens
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc()]
                name: String,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("Failed to parse edoc expression"));

        // Missing constants file: fatal error tokens
        let args = args_from(quote! { from = no_such_file });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("x", APP))]
                name: String,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("Failed to read constants file"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn consts_file_path_maps_module_path_to_file() {
        let base = PathBuf::from("/base");
        let path: syn::Path = syn::parse_quote!(impls::sub::consts);
        assert_eq!(
            consts_file_path(&base, path),
            PathBuf::from("/base/impls/sub/consts.rs")
        );
    }

    #[test]
    fn replace_edoc_attrs_replaces_only_edoc_occurrences() {
        let mut item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("a", CONST))]
                #[doc = "keep me"]
                a: u8,
                #[edoc(("b", CONST))]
                b: u8,
            }
        };
        replace_edoc_attrs(
            &mut item,
            &["doc a".into(), "doc b".into(), "unused".into()],
        );
        let rendered = item.to_token_stream().to_string();
        assert!(rendered.contains("# [doc = \"doc a\"]"));
        assert!(rendered.contains("# [doc = \"doc b\"]"));
        assert!(rendered.contains("# [doc = \"keep me\"]"));
        assert!(!rendered.contains("edoc"));
        // Missing docs leave the edoc attribute untouched
        let mut item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("a", CONST))]
                a: u8,
            }
        };
        replace_edoc_attrs(&mut item, &[]);
        assert!(item.to_token_stream().to_string().contains("edoc"));
    }

    #[test]
    fn resolve_consts_errors_on_invalid_file() {
        let env = env();
        let root = std::env::temp_dir().join("rsmack-edoc-invalid");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("bad.rs"), "not rust at all $$$").expect("write bad file");
        assert!(resolve_consts(&root.join("bad.rs"), &env).is_err());
        fs::remove_dir_all(&root).ok();
    }
}
