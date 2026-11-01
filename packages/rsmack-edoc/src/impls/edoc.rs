//! # Edoc Procedural Macro
//!
//! This module implements a procedural macro called `edoc` (documentation macro) that
//! generates documentation for struct fields by concatenating string literals and constants.
//!
//! ## Overview
//!
//! The `edoc` macro processes struct fields with `#[edoc]` attributes, evaluates expressions
//! containing string literals, constant references and (mini) const expressions, and replaces
//! them with standard `#[doc]` attributes containing the concatenated documentation strings.
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! // In constants.rs
//! const PREFIX: &str = "MyApp";
//! const VERSION: &str = concat!("1.", "0");
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
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};
use syn::{spanned::Spanned, visit::Visit};

use crate::impls::const_eval::{self, Consts, EvalResult};

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
/// 2. Resolving constants from the specified module (memoized per constants file)
/// 3. Evaluating tuple expressions in `edoc` attributes, accumulating **all** errors
/// 4. Replacing `edoc` attributes with `doc` attributes containing concatenated strings
pub fn exec(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> TokenStream {
    env.finish_result(run(args, item, env))
}

fn run(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let Args { from } = args;
    let consts_file_path = consts_file_path(&package_src_folder(), &from);
    let resolved_consts = resolve_consts(&consts_file_path, env)?;

    // Evaluate every edoc attribute, accumulating all errors instead of failing on
    // the first one: one pass reports every broken expression of the struct at once.
    let mut evaluated_edoc_fields: Vec<Option<String>> = vec![];
    for field in &item.fields {
        for attr in field.attrs.iter().filter(|a| a.path().is_ident("edoc")) {
            match attr.parse_args::<syn::Expr>() {
                Err(e) => {
                    env.logr
                        .emit_error(attr.span(), format!("Failed to parse edoc expression: {e}"));
                    // The attribute is dropped so the emitted struct does not carry an
                    // unknown `edoc` attribute anymore.
                    evaluated_edoc_fields.push(None);
                }
                Ok(syn::Expr::Tuple(tup)) => evaluated_edoc_fields.push(Some(
                    evaluate_tuple_elems(&tup.elems, &resolved_consts.consts, env).concat(),
                )),
                Ok(_) => {
                    env.logr.emit_error(
                        attr.span(),
                        "Only Tuple supported, maybe you are missing a second element",
                    );
                    evaluated_edoc_fields.push(None);
                }
            }
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
/// String literals are used as-is; everything else goes through the mini const
/// evaluator ([`const_eval`]), which handles const idents, arithmetic, `concat!` and
/// `env!`. Failures emit a non-fatal collected error and the element is skipped, so a
/// single broken element does not hide the other errors of the struct.
fn evaluate_tuple_elems(
    elems: &syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>,
    resolved_consts: &Consts,
    env: &ExecEnv,
) -> Vec<String> {
    let mut evaluated_elems: Vec<String> = vec![];
    for elem in elems {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit),
            ..
        }) = elem
        {
            // Fast path: string literals keep their precise spans and need no evaluation.
            evaluated_elems.push(lit.value());
            continue;
        }
        match const_eval::eval(elem, resolved_consts, &mut |ident| {
            Err(format!("Unresolved const ident `{ident}`"))
        }) {
            Ok(value) => evaluated_elems.push(value),
            Err(reason) => env.logr.emit_error(elem.span(), reason),
        }
    }
    evaluated_elems
}

/// Converts a module path to a file system path relative to `src_folder`
///
/// Transforms a Rust module path (e.g., `my_module::submodule`) to a file system path
/// by converting `::` to `/` and appending `.rs` extension.
#[must_use]
fn consts_file_path(src_folder: &Path, path: &syn::Path) -> PathBuf {
    src_folder.join(format!(
        "{}.rs",
        path.to_token_stream()
            .to_string()
            .replace(' ', "")
            .replace("::", "/")
    ))
}

/// Replaces every `edoc` attribute of the struct fields with generated `#[doc]`
/// attributes.
///
/// * `Some(doc)` — the attribute becomes `#[doc = doc]`.
/// * `Some(None)` — the evaluation failed and an error was collected: the attribute is
///   dropped so the emitted struct does not carry an unknown `edoc` attribute.
/// * exhausted entries — the remaining `edoc` attributes are left untouched.
fn replace_edoc_attrs(item: &mut syn::ItemStruct, docs: &[Option<String>]) {
    let mut doc_idx = 0;
    for field in &mut item.fields {
        let mut attr_idx = 0;
        while attr_idx < field.attrs.len() {
            if !field.attrs[attr_idx].path().is_ident("edoc") {
                attr_idx += 1;
                continue;
            }
            match docs.get(doc_idx) {
                Some(Some(doc_str)) => {
                    field.attrs[attr_idx] = syn::parse_quote! { #[doc = #doc_str] };
                    attr_idx += 1;
                }
                Some(None) => {
                    field.attrs.remove(attr_idx);
                }
                None => break,
            }
            doc_idx += 1;
        }
    }
}

/// The parsed and evaluated content of a constants file.
#[derive(Debug, Default)]
struct ConstFile {
    /// Successfully resolved constants.
    consts: Consts,
    /// Non-fatal issues (unsupported expressions, cycles, ...), re-emitted on every
    /// use of the cached file so every annotated struct reports them.
    issues: Vec<String>,
}

/// Process-global memoization of constants-file resolution: the same `consts.rs` is
/// shared by every annotated struct of a crate, so N structs cost one parse.
///
/// The cache is keyed by path and validated against `(mtime, len)`, which makes edits
/// picked up without any invalidation API.
static CONSTS_CACHE: OnceLock<Mutex<HashMap<PathBuf, CachedConsts>>> = OnceLock::new();

/// The process-global cache, initialized on first use.
fn consts_cache() -> &'static Mutex<HashMap<PathBuf, CachedConsts>> {
    CONSTS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug)]
struct CachedConsts {
    mtime: SystemTime,
    len: u64,
    data: Arc<ConstFile>,
}

/// Reads, parses, evaluates and memoizes the constants of a source file
///
/// The file is parsed as a real Rust AST (`syn::File`), then every `const` item —
/// including those nested in modules — is collected with [`ConstCollector`] and
/// evaluated with [`const_eval`].
///
/// # Supported Constant Expressions
/// * literals: `&str`, `bool`, `&[u8]`, integers, floats, `char`
/// * arithmetic, bitwise, shift and comparison expressions
/// * `concat!(...)`, `env!(...)`, `option_env!(...)`
/// * references to other constants of the same file (no cycles)
///
/// Unsupported constant expressions are collected as non-fatal issues and re-emitted
/// on every use of the file.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] if the file cannot be read or parsed.
fn resolve_consts(consts_file_path: &Path, env: &ExecEnv) -> syn::Result<Arc<ConstFile>> {
    let source = std::fs::read_to_string(consts_file_path).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to read constants file {}: {e}",
            consts_file_path.display()
        ))
    })?;

    // Cache validation key: files whose metadata cannot be read are simply re-parsed.
    let cache_key = std::fs::metadata(consts_file_path)
        .ok()
        .and_then(|metadata| {
            metadata
                .modified()
                .ok()
                .map(|mtime| (mtime, metadata.len()))
        });

    let cached_hit = cache_key.as_ref().and_then(|(mtime, len)| {
        let cache = consts_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cached = cache.get(consts_file_path)?;
        (cached.mtime == *mtime && cached.len == *len).then(|| {
            for issue in &cached.data.issues {
                env.logr.emit_error(Span::call_site(), issue.clone());
            }
            Arc::clone(&cached.data)
        })
    });
    if let Some(data) = cached_hit {
        return Ok(data);
    }

    let (consts, issues) = parse_consts_source(&source).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to parse constants file {}: {e:?}",
            consts_file_path.display()
        ))
    })?;
    let data = Arc::new(ConstFile { consts, issues });
    for issue in &data.issues {
        env.logr.emit_error(Span::call_site(), issue.clone());
    }
    if let Some((mtime, len)) = cache_key {
        cache_insert(consts_file_path, mtime, len, Arc::clone(&data));
    }
    Ok(data)
}

/// Parse and evaluate a constants **source string** into its resolved constants and
/// non-fatal issues.
///
/// The pure core of [`resolve_consts`]: arbitrary input never panics, it either fails
/// to parse (reported as a [`syn::Error`]) or reports evaluation issues.
///
/// # Errors
/// Returns the [`syn::Error`] of `syn::parse_file` when the source is not valid Rust.
pub fn parse_consts_source(source: &str) -> syn::Result<(Consts, Vec<String>)> {
    let file = syn::parse_file(source)?;
    let mut collector = ConstCollector::default();
    collector.visit_file(&file);
    let ConstFile { consts, issues } = collector.finish();
    Ok((consts, issues))
}

/// Insert a freshly parsed constants file into the process-global memo.
fn cache_insert(path: &Path, mtime: SystemTime, len: u64, data: Arc<ConstFile>) {
    let mut cache = consts_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    cache.insert(path.to_path_buf(), CachedConsts { mtime, len, data });
}

/// A [`Visit`]or collecting the raw constant expressions of a file
#[derive(Default)]
struct ConstCollector {
    /// Constants in source order (deterministic issue reporting).
    raw: Vec<(String, syn::Expr)>,
}

impl ConstCollector {
    /// Evaluate every collected constant, resolving const-to-const references lazily
    /// and detecting cycles.
    fn finish(self) -> ConstFile {
        let mut file = ConstFile::default();
        let mut resolved: HashMap<String, EvalResult> = HashMap::new();
        let mut stack: Vec<String> = Vec::new();
        let raw = &self.raw;
        let mut resolve =
            |name: &str| -> EvalResult { resolve_const(name, raw, &mut resolved, &mut stack) };
        for (name, expr) in raw {
            match const_eval::eval(expr, &file.consts, &mut resolve) {
                Ok(value) => {
                    file.consts.insert(name.clone(), value);
                }
                Err(reason) => file
                    .issues
                    .push(format!("Failed to const-evaluate `{name}`: {reason}")),
            }
        }
        file
    }
}

/// Resolve a const name through the raw expressions, memoizing results and rejecting cycles.
fn resolve_const(
    name: &str,
    raw: &[(String, syn::Expr)],
    resolved: &mut HashMap<String, EvalResult>,
    stack: &mut Vec<String>,
) -> EvalResult {
    if let Some(cached) = resolved.get(name) {
        return cached.clone();
    }
    if stack.iter().any(|in_progress| in_progress == name) {
        return Err(format!("Cyclic const dependency involving `{name}`"));
    }
    let Some((_, expr)) = raw.iter().find(|(raw_name, _)| raw_name == name) else {
        return Err(format!("Unresolved const ident `{name}`"));
    };
    stack.push(name.to_string());
    let empty_consts = Consts::new();
    let outcome = const_eval::eval(expr, &empty_consts, &mut |nested| {
        resolve_const(nested, raw, resolved, stack)
    });
    stack.pop();
    resolved.insert(name.to_string(), outcome.clone());
    outcome
}

impl Visit<'_> for ConstCollector {
    fn visit_item_const(&mut self, item_const: &syn::ItemConst) {
        let const_name = item_const.ident.to_string();
        self.raw.push((const_name, (*item_const.expr).clone()));
        syn::visit::visit_item_const(self, item_const);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;
    use std::fs;

    fn env() -> ExecEnv {
        ExecEnv::builder("rsmack_edoc::impls").build()
    }

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    /// Run `exec` with `CARGO_MANIFEST_DIR` pointing at `root`.
    fn exec_in(root: &Path, args: Args, item: syn::ItemStruct) -> String {
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", root) };
        exec(args, item, &env()).to_string()
    }

    /// Serialize the tests mutating process-wide environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_consts_file(root: &Path, name: &str, content: &str) {
        fs::create_dir_all(root.join("src")).expect("create src");
        fs::write(root.join("src").join(format!("{name}.rs")), content).expect("write consts");
    }

    /// Constants file exercising every supported const expression kind, plus one
    /// unsupported expression (`NOT_A_LITERAL`).
    fn setup_consts_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        write_consts_file(
            &root,
            "consts",
            r#"
            pub const APP: &str = "myapp";
            pub const VERSION: &str = "1.2.3";
            pub const VERBOSE: bool = true;
            pub const MAGIC: &[u8] = b"ab";
            pub const GREETING: &str = concat!("hello", " ", "world");
            pub const MAJOR: i32 = 1;
            pub const TAG: &str = concat!("v", MAJOR);
            mod nested {
                pub const DEEP: &str = "deep";
            }
            pub const NOT_A_LITERAL: &str = vec![1];
            pub fn not_a_const() {}
            "#,
        );
        root
    }

    #[test]
    fn exec_resolves_consts_and_preserves_fields() {
        let _guard = env_guard();
        let root = setup_consts_root("rsmack-edoc-tests");

        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            #[derive(Debug)]
            struct Client {
                /// Overwritten doc
                #[edoc(("App: ", APP, " v", VERSION))]
                name: String,
                #[edoc(("Verbose: ", VERBOSE))]
                verbose: bool,
                #[edoc(("Greeting: ", GREETING))]
                greeting: String,
                #[edoc(("Tag: ", TAG))]
                tag: String,
                #[edoc(("Deep: ", nested::DEEP))]
                deep: String,
                #[edoc(("Magic: ", MAGIC))]
                magic: String,
                #[serde(rename = "n")]
                #[edoc(("Fallback ", "plain"))]
                other: u8,
                #[edoc(("Missing: ", NOPE))]
                missing: String,
                #[edoc(("Arith: ", 1 + 1))]
                arith: String,
                without_edoc: u8,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(
            output.contains("# [doc = \"App: myapp v1.2.3\"]"),
            "unexpected output: {output}"
        );
        assert!(output.contains("# [doc = \"Verbose: true\"]"));
        assert!(output.contains("# [doc = \"Greeting: hello world\"]"));
        assert!(output.contains("# [doc = \"Tag: v1\"]"));
        assert!(output.contains("# [doc = \"Deep: deep\"]"));
        assert!(output.contains("# [doc = \"Magic: [97, 98]\"]"));
        assert!(output.contains("# [doc = \"Arith: 2\"]"));
        // non-edoc attributes are preserved
        assert!(output.contains("serde (rename"));
        assert!(output.contains("derive (Debug)"));
        // fields without edoc attribute are kept untouched
        assert!(output.contains("without_edoc : u8"));
        // unresolved const emits a collected compile_error, other fields keep their docs
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unresolved const ident"));
        assert!(output.contains("# [doc = \"Missing: \"]"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn exec_reports_unsupported_consts_without_aborting() {
        let _guard = env_guard();
        let root = setup_consts_root("rsmack-edoc-unsupported");

        // Unsupported const expression in the file: collected error, but other consts still resolve
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("App: ", APP))]
                name: String,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(output.contains("Failed to const-evaluate `NOT_A_LITERAL`"));
        assert!(output.contains("# [doc = \"App: myapp\"]"));

        // Unsupported tuple element: collected error, other elements keep resolving
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("literal ok ", APP, vec![1]))]
                name: String,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(output.contains("Unsupported const macro"));
        assert!(output.contains("# [doc = \"literal ok myapp\"]"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn exec_drops_broken_attributes_and_continues() {
        let _guard = env_guard();
        let root = setup_consts_root("rsmack-edoc-broken-attrs");

        // Non-tuple edoc expression: collected error, attribute dropped
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc("not a tuple")]
                name: String,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(output.contains("Only Tuple supported"));
        assert!(!output.contains("# [edoc"));

        // Malformed edoc expression: collected error, attribute dropped
        let args = args_from(quote! { from = consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc()]
                name: String,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(output.contains("Failed to parse edoc expression"));
        assert!(!output.contains("# [edoc"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn exec_missing_consts_file_is_fatal() {
        let _guard = env_guard();
        let root = setup_consts_root("rsmack-edoc-missing-file");

        let args = args_from(quote! { from = no_such_file });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("x", APP))]
                name: String,
            }
        };
        let output = exec_in(&root, args, item);
        assert!(output.contains("Failed to read constants file"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn consts_cache_picks_up_file_changes() {
        let _guard = env_guard();
        let root = std::env::temp_dir().join("rsmack-edoc-cache");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        write_consts_file(&root, "cache_consts", "pub const ONE: &str = \"1\";\n");

        let args = args_from(quote! { from = cache_consts });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("One: ", ONE))]
                one: String,
            }
        };
        let first = exec_in(
            &root,
            args_from(quote! { from = cache_consts }),
            item.clone(),
        );
        assert!(first.contains("# [doc = \"One: 1\"]"));

        // Change the file content (different length): the memoized entry is invalidated
        write_consts_file(&root, "cache_consts", "pub const ONE: &str = \"one!!\";\n");
        let second = exec_in(&root, args, item);
        assert!(second.contains("# [doc = \"One: one!!\"]"));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn const_references_are_memoized_and_missing_ones_reported() {
        let env = env();
        let dir = std::env::temp_dir().join("rsmack-edoc-refs");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(
            dir.join("refs.rs"),
            "const X: &str = \"x\";\nconst A: &str = concat!(X, X);\nconst B: &str = MISSING;\n",
        )
        .expect("write refs file");
        let resolved = resolve_consts(&dir.join("refs.rs"), &env).expect("resolve");
        assert_eq!(resolved.consts.get("A").expect("A resolved"), "xx");
        // `B` references an unknown const: reported, and the memoized resolution is reused
        assert!(
            resolved.issues.len() == 1
                && resolved.issues[0].contains("Unresolved const ident `MISSING`")
        );
        let again = resolve_consts(&dir.join("refs.rs"), &env).expect("resolve again");
        assert_eq!(again.consts.get("A").expect("A resolved"), "xx");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cyclic_consts_report_an_issue() {
        let env = env();
        let dir = std::env::temp_dir().join("rsmack-edoc-cycle");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create dir");
        fs::write(
            dir.join("cycle.rs"),
            "pub const A: &str = B;\npub const B: &str = A;\n",
        )
        .expect("write cycle file");
        let resolved = resolve_consts(&dir.join("cycle.rs"), &env).expect("resolve");
        assert!(
            resolved
                .issues
                .iter()
                .any(|i| i.contains("Cyclic const dependency"))
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn consts_file_path_maps_module_path_to_file() {
        let base = PathBuf::from("/base");
        let path: syn::Path = syn::parse_quote!(impls::sub::consts);
        assert_eq!(
            consts_file_path(&base, &path),
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
            &[
                Some("doc a".into()),
                Some("doc b".into()),
                Some("unused".into()),
            ],
        );
        let rendered = item.to_token_stream().to_string();
        assert!(rendered.contains("# [doc = \"doc a\"]"));
        assert!(rendered.contains("# [doc = \"doc b\"]"));
        assert!(rendered.contains("# [doc = \"keep me\"]"));
        assert!(!rendered.contains("edoc"));

        // A None entry drops the attribute entirely
        let mut item: syn::ItemStruct = syn::parse_quote! {
            struct S {
                #[edoc(("a", CONST))]
                a: u8,
                #[edoc(("b", CONST))]
                b: u8,
            }
        };
        replace_edoc_attrs(&mut item, &[Some("doc a".into()), None]);
        let rendered = item.to_token_stream().to_string();
        assert!(!rendered.contains("edoc"));
        assert!(rendered.contains("# [doc = \"doc a\"]"));
        assert!(rendered.contains("b : u8"));

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
    fn parse_consts_source_is_total() {
        // Arbitrary garbage never panics: it either fails to parse or yields issues.
        assert!(parse_consts_source("this is not rust at all $$$").is_err());

        let (consts, issues) = parse_consts_source(
            "const A: &str = B;\nconst B: &str = A;\nconst OK: &str = concat!(\"x\", 1 + 1);\n",
        )
        .expect("valid rust source");
        assert_eq!(consts.get("OK").expect("OK resolved"), "x2");
        assert_eq!(issues.len(), 2);
        assert!(issues[0].contains("Cyclic const dependency"));
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
