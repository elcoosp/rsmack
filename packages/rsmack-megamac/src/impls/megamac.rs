//! Procedural macro for generating other procedural macros with documentation and error
//! handling.
//!
//! This module provides the `megamac` procedural macro that can generate different types
//! of procedural macros (function-like, attribute, derive) with automatic documentation
//! generation and proper error handling.
//!
//! The v2 protocol is **signature-driven**: the `exec` signature of the implementation
//! decides how the payload is handled —
//!
//! * `exec(args: Args, env: &ExecEnv)` → darling meta parsing;
//! * `exec(input: TokenStream, env: &ExecEnv)` → the raw payload tokens, unlocking
//!   DSL-style macros (`sql!(SELECT ...)`, `html!(<div> ...)`);
//! * the parsed item type of `Attr`/`Derive` macros stays inferred from the signature.
//!
//! The impl file is parsed **once per process** (memoized by `(path, mtime, len)` plus
//! the configuration affecting the extraction), and the extracted analysis (payload
//! mode, docs) is stored as plain data — token trees carry non-`Send` spans and could
//! not cross the cache anyway. Warm expansions (rust-analyzer re-checks, test loops)
//! pay a metadata stat instead of a full read + parse.

use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use rsmack_utils::megamac::ExecEnv;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use strum::Display;
use syn::{Expr, ExprLit, FnArg, Ident, Item, ItemFn, Lit, Meta, MetaNameValue, Type};

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
    /// No generated documentation at all.
    None,
}

impl MacroKind {
    /// Converts an ident to a [`MacroKind`].
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
    /// Converts an ident to a [`DocSource`].
    ///
    /// # Errors
    /// Returns an error if the identifier doesn't match `readme`, `inline` or `none`.
    fn from_ident(value: &Ident) -> syn::Result<Self> {
        match value.to_string().as_str() {
            "readme" => Ok(Self::Readme),
            "inline" => Ok(Self::Inline),
            "none" => Ok(Self::None),
            _ => Err(syn::Error::new(
                value.span(),
                format!("Unsupported doc source `{value}`, expected `readme`, `inline` or `none`"),
            )),
        }
    }
}

/// Arguments for configuring the megamac macro generation.
#[derive(Debug, FromMeta)]
pub struct Args {
    /// The type of procedural macro to generate as an ident
    pub(crate) kind: Ident,

    /// Name of the macro to generate as an ident
    pub(crate) name: Ident,

    /// Ident of the implementations module, `impls` by default
    #[darling(default)]
    pub(crate) impls: Option<Ident>,

    /// Ident of the args struct, `Args` by default (darling payload mode only)
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

/// How the generated entrypoint feeds the payload to `exec`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum PayloadMode {
    /// Darling meta parsing of the documented `Args` struct.
    #[default]
    Darling,
    /// The raw payload tokens are forwarded untouched.
    Raw,
}

/// Documentation information for a field in the macro arguments struct.
///
/// Everything is plain data: token trees carry non-`Send` spans and cannot cross the
/// process-global cache.
#[derive(Debug, Clone)]
pub(crate) struct FieldDoc {
    /// The field identifier
    pub(crate) ident: String,
    /// The field documentation string, if present
    pub(crate) doc: Option<String>,
    /// The compacted field type rendering (spaces stripped)
    pub(crate) ty: String,
    /// Whether the field is optional (`Option<T>` or `#[darling(default)]`)
    pub(crate) optional: bool,
}

/// The configuration-dependent extraction of an impl file, as plain data.
#[derive(Debug, Default)]
pub(crate) struct ImplAnalysis {
    /// Payload protocol inferred from the `exec` signature.
    mode: PayloadMode,
    /// Args struct ident used for the docs lookup (darling mode only).
    args_ident: String,
    /// Extracted field docs (darling `Func`/`Attr` only).
    fields: Vec<FieldDoc>,
    /// The struct-level doc lines of the args struct (intro paragraph).
    intro: Vec<String>,
    /// Fatal problem of the analysis (missing args struct, contradictory config).
    fatal: Option<String>,
    /// Non-fatal problems, re-emitted on every use of the cached analysis.
    issues: Vec<String>,
}

/// The process-global memoization of impl-file analysis: the same
/// `<src>/<impls>/<name>.rs` is re-read on every expansion of the *declaring* macro
/// (rust-analyzer re-expansions, test loops), so the read + parse + extraction are
/// memoized per `(path, mtime, len, kind, args override)`.
static IMPL_CACHE: OnceLock<Mutex<HashMap<PathBuf, CachedImplAnalysis>>> = OnceLock::new();

#[derive(Debug)]
struct CachedImplAnalysis {
    mtime: std::time::SystemTime,
    len: u64,
    kind: String,
    args_override: Option<String>,
    analysis: Arc<ImplAnalysis>,
}

fn impl_cache() -> &'static Mutex<HashMap<PathBuf, CachedImplAnalysis>> {
    IMPL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn run(args: &Args, env: &ExecEnv, src_folder: &Path) -> syn::Result<TokenStream> {
    let kind = MacroKind::from_ident(&args.kind).map_err(|e| env.logr.error(e.span(), e))?;
    let doc_source = args.doc_source().map_err(|e| env.logr.error(e.span(), e))?;
    let impls_ident = args.impls_ident();
    let name = &args.name;
    let analysis = resolve_analysis(args, env, src_folder, &impls_ident, kind)?;
    if let Some(fatal) = &analysis.fatal {
        return Err(env.logr.call_site_error(fatal.clone()));
    }

    // Format field documentation for inclusion in the generated macro docs
    let formatted_fields_doc: Vec<_> = analysis
        .fields
        .iter()
        .map(|fd| {
            let optional = if fd.optional { " *(optional)*" } else { "" };
            let template = format!(
                "* `{}` - {}\n  + type: [`{}`]{}",
                fd.ident,
                fd.doc.as_deref().unwrap_or("Not documented"),
                fd.ty,
                optional,
            );
            quote! { #[doc = #template]}
        })
        .collect();

    // The struct-level doc of the `Args` struct becomes the intro paragraph.
    let intro_doc = analysis
        .intro
        .iter()
        .map(|line| quote! { #[doc = #line] })
        .collect::<Vec<_>>();

    let name_str = name.to_string();
    let kind_str = kind.to_string();

    // Generate the main documentation for the macro
    let readme_doc = match doc_source {
        DocSource::Readme => quote! { #[doc = include_str!("../README.md")] },
        DocSource::Inline | DocSource::None => quote! {},
    };
    let params_docs = (doc_source != DocSource::None).then(|| {
        // The parameters section only exists when the macro actually takes parameters.
        let params = (!analysis.fields.is_empty())
            .then(|| quote! { #[doc = "# Parameters"] #(#formatted_fields_doc)* });
        quote! {
            #readme_doc
            #[doc = concat!(#name_str, " procedural macro (", #kind_str, ").")]
            #(#intro_doc)*
            #[doc = ""]
            #params
        }
    });

    let macro_impl = generate_entrypoint(kind, analysis.mode, name, &impls_ident);

    Ok(quote! {
        #params_docs
        #macro_impl
    })
}

/// Generate the proc-macro entrypoint of a megamac-declared macro.
///
/// The implementation routes through the shared rsmack-utils drivers: darling meta
/// parsing, item parsing and diagnostics flushing are single-sourced, and the payload
/// flavor (darling vs raw) follows the `exec` signature of the implementation.
fn generate_entrypoint(
    kind: MacroKind,
    mode: PayloadMode,
    name: &Ident,
    impls_ident: &Ident,
) -> TokenStream {
    let name_str = name.to_string();
    let exec_path = quote! { crate::#impls_ident::#name::exec };
    match (kind, mode) {
        (MacroKind::Derive, _) => {
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
        (MacroKind::Func, PayloadMode::Raw) => quote! {
            #[proc_macro]
            pub fn #name(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
                rsmack_utils::exec::func_raw_entrypoint(
                    args.into(),
                    concat!(std::module_path!(), "::", #name_str),
                    |__input, __env| #exec_path(__input, __env),
                )
                .into()
            }
        },
        (MacroKind::Func, PayloadMode::Darling) => quote! {
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
        (MacroKind::Attr, PayloadMode::Raw) => {
            // The parsed item type is inferred from the `exec` signature of the
            // implementation: no receiver configuration is needed.
            quote! {
                #[proc_macro_attribute]
                pub fn #name(
                    attr: proc_macro::TokenStream,
                    item: proc_macro::TokenStream,
                ) -> proc_macro::TokenStream {
                    rsmack_utils::exec::attr_raw_entrypoint(
                        attr.into(),
                        item.into(),
                        concat!(std::module_path!(), "::", #name_str),
                        |__input, __item, __env| #exec_path(__input, __item, __env),
                    )
                    .into()
                }
            }
        }
        (MacroKind::Attr, PayloadMode::Darling) => {
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
    }
}

/// Resolve the analysis of the impl file, through the process-global cache.
///
/// Cached issues are re-emitted and a cached fatal is returned, so every expansion of
/// the declaring macro reports the same diagnostics as the cold one.
///
/// # Errors
/// Returns a prefixed [`syn::Error`] when the file cannot be read or parsed, or when
/// the stored analysis is fatal.
fn resolve_analysis(
    args: &Args,
    env: &ExecEnv,
    src_folder: &Path,
    impls_ident: &Ident,
    kind: MacroKind,
) -> syn::Result<Arc<ImplAnalysis>> {
    let macro_impl_file_path = impl_file_path(src_folder, impls_ident, &args.name);
    let cache_key = rsmack_utils::fs::staleness_key(&macro_impl_file_path);
    let kind_str = kind.to_string();
    let args_override = args.arguments.as_ref().map(ToString::to_string);

    let cached_hit = cache_key.as_ref().and_then(|(mtime, len)| {
        let cache = impl_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cached = cache.get(&macro_impl_file_path)?;
        (cached.mtime == *mtime
            && cached.len == *len
            && cached.kind == kind_str
            && cached.args_override == args_override)
            .then(|| Arc::clone(&cached.analysis))
    });
    if let Some(analysis) = cached_hit {
        for issue in &analysis.issues {
            env.logr.emit_error(Span::call_site(), issue.clone());
        }
        return Ok(analysis);
    }

    let macro_impl_src = std::fs::read_to_string(&macro_impl_file_path).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to get macro_impl_src of {} at {}: {e}",
            args.name,
            macro_impl_file_path.display()
        ))
    })?;

    let impl_file = syn::parse_file(&macro_impl_src).map_err(|e| {
        env.logr.call_site_error(format!(
            "Failed to parse macro_impl_src {}, this may happen for no real reason in your IDE, check that your project still builds with cargo: {e:?}",
            args.name
        ))
    })?;

    let mut analysis = analyze_impl(&impl_file, args, kind);
    // A raw payload cannot carry darling args: `args = ...` is a configuration bug.
    if analysis.mode == PayloadMode::Raw && args_override.is_some() {
        analysis.fatal = Some(format!(
            "`args = {}` is set but `exec` takes a raw `TokenStream` payload: remove `args` or change the signature to take a documented args struct",
            args.arguments.as_ref().expect("checked above")
        ));
    }

    let analysis = Arc::new(analysis);
    for issue in &analysis.issues {
        env.logr.emit_error(Span::call_site(), issue.clone());
    }
    if let Some((mtime, len)) = cache_key {
        let mut cache = impl_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.insert(
            macro_impl_file_path,
            CachedImplAnalysis {
                mtime,
                len,
                kind: kind_str,
                args_override,
                analysis: Arc::clone(&analysis),
            },
        );
    }
    if let Some(fatal) = &analysis.fatal {
        return Err(env.logr.call_site_error(fatal.clone()));
    }
    Ok(analysis)
}

/// Path of the implementation module of `name`: `<src_folder>/<impls>/<name>.rs`.
#[must_use]
fn impl_file_path(src_folder: &Path, impls_ident: &Ident, name: &Ident) -> PathBuf {
    src_folder
        .join(impls_ident.to_string())
        .join(format!("{name}.rs"))
}

/// Find the `exec` function of the implementation, if any.
fn find_exec_fn(impl_file: &syn::File) -> Option<&ItemFn> {
    impl_file.items.iter().find_map(|item| match item {
        Item::Fn(item_fn) if item_fn.sig.ident == "exec" => Some(item_fn),
        _ => None,
    })
}

/// Whether `ty` is a (possibly referenced) path whose last segment is `expected`.
fn path_last_segment(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(type_ref) => path_last_segment(&type_ref.elem),
        _ => None,
    }
}

/// Whether `ty` is `&ExecEnv` (the trailing environment parameter of `exec`).
fn is_exec_env_ref(ty: &Type) -> bool {
    path_last_segment(ty).as_deref() == Some("ExecEnv")
}

/// Infer the payload protocol from the `exec` signature of the implementation.
///
/// The trailing `&ExecEnv` parameter is stripped, then:
///
/// * `Func` — the single remaining parameter decides: a `TokenStream` payload means raw
///   mode, anything else is the darling `Args` type (its ident feeds the docs lookup);
/// * `Attr` — two remaining parameters: same payload rule for the first one, the item
///   type stays inferred;
/// * `Derive` — one remaining item parameter, no payload at all.
///
/// When no `exec` function is found the legacy darling/`Args` default is assumed: the
/// file will not compile anyway, and reporting it here would only duplicate rustc.
#[must_use]
fn analyze_exec(impl_file: &syn::File, args: &Args, kind: MacroKind) -> (PayloadMode, String) {
    let default_ident = args.args_ident().to_string();
    let mut result = (PayloadMode::Darling, default_ident.clone());
    let Some(item_fn) = find_exec_fn(impl_file) else {
        return result;
    };
    let mut params: Vec<&Type> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => Some(pat_type.ty.as_ref()),
            FnArg::Receiver(_) => None,
        })
        .collect();
    if params.last().is_some_and(|ty| is_exec_env_ref(ty)) {
        params.pop();
    }
    let payload = match (kind, params.first()) {
        (MacroKind::Derive, _) | (_, None) => None,
        (MacroKind::Func | MacroKind::Attr, Some(ty)) => Some(*ty),
    };
    let Some(payload_ty) = payload else {
        return result;
    };
    if path_last_segment(payload_ty).as_deref() == Some("TokenStream") {
        result = (PayloadMode::Raw, default_ident);
    } else {
        // The args ident follows the signature (e.g. `MyArgs`), unless the `args`
        // option of `megamac!` overrides it explicitly.
        let ident = args.arguments.clone().map(|ident| ident.to_string());
        result.1 = ident
            .unwrap_or_else(|| path_last_segment(payload_ty).unwrap_or_else(|| result.1.clone()));
    }
    result
}

/// Analyze the impl file: payload protocol, args docs, intro, problems.
///
/// Problems are **collected** into the analysis (fatal + issues) instead of being
/// emitted, so the cold and cached paths report identical diagnostics.
#[must_use]
fn analyze_impl(impl_file: &syn::File, args: &Args, kind: MacroKind) -> ImplAnalysis {
    let (mode, args_ident) = analyze_exec(impl_file, args, kind);
    let mut analysis = ImplAnalysis {
        mode,
        args_ident,
        ..ImplAnalysis::default()
    };
    // Darling mode documents the `Args` struct; raw mode and derives have no parameter
    // section (a derive receives no attribute payload, a raw payload has no Args).
    if mode == PayloadMode::Darling && matches!(kind, MacroKind::Func | MacroKind::Attr) {
        let args_ident = Ident::new(&analysis.args_ident, args.name.span());
        match args_fields_doc(impl_file, &args_ident, &args.impls_ident()) {
            Ok((fields, intro)) => {
                analysis.fields = fields;
                analysis.intro = intro;
            }
            Err(fatal) => analysis.fatal = Some(fatal),
        }
    }
    analysis
}

/// Whether the field type is `Option<...>`.
fn is_option_type(ty: &Type) -> bool {
    path_last_segment(ty).as_deref() == Some("Option")
}

/// Whether the field carries a `#[darling(default)]`-style attribute.
fn has_darling_default(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("darling") {
            return false;
        }
        match &attr.meta {
            // `#[darling(default)]`
            Meta::List(meta_list) => darling::ast::NestedMeta::parse_meta_list(
                meta_list.tokens.clone(),
            )
            .is_ok_and(|nested| {
                nested.iter().any(|item| match item {
                    darling::ast::NestedMeta::Meta(Meta::Path(path)) => path.is_ident("default"),
                    darling::ast::NestedMeta::Meta(Meta::List(inner)) => {
                        inner.path.is_ident("default")
                    }
                    darling::ast::NestedMeta::Meta(Meta::NameValue(nv)) => {
                        nv.path.is_ident("default")
                    }
                    // Recoverable parse variants and literals carry no `default` flag.
                    _ => false,
                })
            }),
            _ => false,
        }
    })
}

/// Extracts documentation for all fields in the macro arguments struct, plus the
/// struct-level doc lines (the intro paragraph of the generated macro).
///
/// # Errors
/// Returns the (unprefixed) fatal message when the args struct cannot be found.
fn args_fields_doc(
    impl_file: &syn::File,
    args_ident: &Ident,
    impls_ident: &Ident,
) -> Result<(Vec<FieldDoc>, Vec<String>), String> {
    let Some((args_struct, struct_attrs)) = impl_file.items.iter().find_map(|item| match item {
        Item::Struct(item_struct) if item_struct.ident == *args_ident => {
            Some((&item_struct.fields, &item_struct.attrs))
        }
        _ => None,
    }) else {
        return Err(format!(
            "Failed to find `{args_ident}` struct in `{impls_ident}`"
        ));
    };

    let fields_named = match args_struct {
        syn::Fields::Named(fields_named) => fields_named,
        // A unit `Args` struct is legitimate: the macro takes no parameters.
        syn::Fields::Unit => return Ok((vec![], struct_doc_lines(struct_attrs))),
        syn::Fields::Unnamed(_) => {
            return Err(format!(
                "Unsupported `{args_ident}` struct: only named fields are documented, received `{}`",
                args_struct.to_token_stream()
            ));
        }
    };

    let fields = fields_named
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
                }) if path.is_ident("doc") => Some(lit_str.value().trim_start().to_string()),
                _ => None,
            });
            FieldDoc {
                ident: f.ident.clone().expect("named field").to_string(),
                doc,
                ty: f.ty.to_token_stream().to_string().replace(' ', ""),
                optional: is_option_type(&f.ty) || has_darling_default(&f.attrs),
            }
        })
        .collect();
    Ok((fields, struct_doc_lines(struct_attrs)))
}

/// The struct-level doc lines of the `Args` struct (empty when absent).
///
/// The doc comment written above `Args` becomes the intro paragraph of the generated
/// macro's documentation.
#[must_use]
fn struct_doc_lines(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }),
                path,
                ..
            }) if path.is_ident("doc") => Some(lit_str.value().trim_start().to_string()),
            _ => None,
        })
        .collect()
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
        ExecEnv::new("rsmack_megamac::megamac")
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
        // `Option<T>` fields are marked optional
        assert!(output.contains("(optional)"));
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    /// The raw payload protocol: `exec(input: TokenStream, env)` generates a
    /// `func_raw_entrypoint` entrypoint without a `# Parameters` section.
    #[test]
    fn exec_detects_the_raw_payload_mode() {
        let root = std::env::temp_dir().join("rsmack-megamac-raw-mode");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("dsl.rs"),
            r"
            use proc_macro2::TokenStream;
            use rsmack_utils::megamac::ExecEnv;

            pub fn exec(input: TokenStream, env: &ExecEnv) -> TokenStream {
                let echoed = input.to_string();
                env.finish(quote::quote! { const PAYLOAD: &str = #echoed; })
            }
            ",
        )
        .expect("write raw impl file");
        let _guard = with_manifest_dir(&root);

        let args = args_from(quote! { kind = "Func", name = "dsl" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("# [proc_macro]"));
        assert!(output.contains("func_raw_entrypoint"));
        assert!(!output.contains("func_entrypoint"));
        assert!(!output.contains("compile_error"));
        // No `Args` struct is required: no parameters section at all.
        assert!(!output.contains("# Parameters"));

        // A raw signature contradicts `args = ...`: fatal, the configuration is broken.
        let args = args_from(quote! { kind = "Func", name = "dsl", args = "SomeArgs" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("raw `TokenStream` payload"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    /// Raw mode also applies to attribute macros (`attr_raw_entrypoint`).
    #[test]
    fn exec_detects_the_raw_attr_mode() {
        let root = std::env::temp_dir().join("rsmack-megamac-raw-attr");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("tagged.rs"),
            r"
            use proc_macro2::TokenStream;
            use rsmack_utils::megamac::ExecEnv;

            pub fn exec(input: TokenStream, item: syn::ItemStruct, env: &ExecEnv) -> TokenStream {
                let _ = input;
                env.finish(quote::quote! { #item })
            }
            ",
        )
        .expect("write raw attr impl file");
        let _guard = with_manifest_dir(&root);

        let args = args_from(quote! { kind = "Attr", name = "tagged" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("# [proc_macro_attribute]"));
        assert!(output.contains("attr_raw_entrypoint"));
        assert!(!output.contains("attr_entrypoint"));
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    /// Custom `Args` idents follow the `exec` signature when no `args` option is set.
    #[test]
    fn exec_follows_custom_args_ident_from_signature() {
        let root = std::env::temp_dir().join("rsmack-megamac-custom-args");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("renamed.rs"),
            r"
            use darling::FromMeta;
            use proc_macro2::TokenStream;
            use rsmack_utils::megamac::ExecEnv;

            /// Renamed args docs
            #[derive(Debug, FromMeta)]
            pub struct RenamedArgs {
                /// Documented field of the renamed struct
                pub demo: String,
            }

            pub fn exec(args: RenamedArgs, env: &ExecEnv) -> TokenStream {
                let _ = args;
                env.finish(quote::quote! {})
            }
            ",
        )
        .expect("write renamed impl file");
        let _guard = with_manifest_dir(&root);

        let args = args_from(quote! { kind = "Func", name = "renamed" });
        let output = exec(&args, &test_env()).to_string();
        assert!(
            output.contains("* `demo` - Documented field of the renamed struct"),
            "got: {output}"
        );
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    /// The struct-level doc of `Args` becomes the intro paragraph.
    #[test]
    fn exec_emits_the_struct_level_doc_as_intro() {
        let root = std::env::temp_dir().join("rsmack-megamac-intro-doc");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("introed.rs"),
            r"
            use darling::FromMeta;
            use proc_macro2::TokenStream;
            use rsmack_utils::megamac::ExecEnv;

            /// Intro line of the macro
            #[derive(Debug, FromMeta)]
            pub struct Args {
                /// Demo field
                pub demo: String,
            }

            pub fn exec(args: Args, env: &ExecEnv) -> TokenStream {
                let _ = args;
                env.finish(quote::quote! {})
            }
            ",
        )
        .expect("write introed impl file");
        let _guard = with_manifest_dir(&root);

        let args = args_from(quote! { kind = "Attr", name = "introed" });
        let output = exec(&args, &test_env()).to_string();
        assert!(output.contains("Intro line of the macro"));
        assert!(output.contains("# Parameters"));
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
            (syn::parse_quote!(none), DocSource::None),
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
    fn exec_doc_none_skips_every_doc() {
        let args = args_from(quote! { kind = "Func", name = "megamac", doc = "none" });
        let output = exec_default(&args);
        assert!(output.contains("# [proc_macro]"));
        assert!(!output.contains("# Parameters"));
        assert!(!output.contains("procedural macro ("));
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
    fn exec_derive_pascal_cases_the_name_and_skips_param_docs() {
        let args = args_from(quote! { kind = "Derive", name = "megamac" });
        let output = exec_default(&args);
        assert!(output.contains("proc_macro_derive"));
        assert!(output.contains("Megamac"));
        assert!(output.contains("derive_entrypoint"));
        // Derives receive no payload: no parameters section.
        assert!(!output.contains("# Parameters"));
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

    #[test]
    fn analyze_exec_defaults_without_exec_fn() {
        let file: syn::File = syn::parse_quote! { pub struct Args; };
        let args = args_from(quote! { kind = "Func", name = "demo" });
        let (mode, args_ident) = analyze_exec(&file, &args, MacroKind::Func);
        assert_eq!(mode, PayloadMode::Darling);
        assert_eq!(args_ident, "Args");
    }

    #[test]
    fn analyze_exec_detects_raw_and_custom_args() {
        let raw: syn::File = syn::parse_quote! {
            pub fn exec(input: proc_macro2::TokenStream, env: &ExecEnv) -> TokenStream {}
        };
        let args = args_from(quote! { kind = "Func", name = "demo" });
        let (mode, _) = analyze_exec(&raw, &args, MacroKind::Func);
        assert_eq!(mode, PayloadMode::Raw);

        let custom: syn::File = syn::parse_quote! {
            pub fn exec(args: MyArgs, item: ItemStruct, env: &rsmack_utils::megamac::ExecEnv) -> TokenStream {}
        };
        let (_, args_ident) = analyze_exec(&custom, &args, MacroKind::Attr);
        assert_eq!(args_ident, "MyArgs");
    }

    #[test]
    fn analyze_exec_env_by_value_and_missing_env() {
        // `env` passed by value: it stays in the params, but the payload inference of
        // the first parameter is unaffected.
        let by_value: syn::File = syn::parse_quote! {
            pub fn exec(args: Args, env: ExecEnv) -> TokenStream {}
        };
        let args = args_from(quote! { kind = "Func", name = "demo" });
        let (mode, args_ident) = analyze_exec(&by_value, &args, MacroKind::Func);
        assert_eq!(mode, PayloadMode::Darling);
        assert_eq!(args_ident, "Args");

        // No env at all: the only parameter is the payload.
        let no_env: syn::File = syn::parse_quote! {
            pub fn exec(input: TokenStream) -> TokenStream {}
        };
        let (mode, _) = analyze_exec(&no_env, &args, MacroKind::Func);
        assert_eq!(mode, PayloadMode::Raw);
    }

    #[test]
    fn has_darling_default_covers_all_meta_shapes() {
        let with_word: syn::File = syn::parse_quote! {
            struct Args { #[darling(default)] demo: Option<String> }
        };
        let Item::Struct(item_struct) = with_word.items.first().expect("struct").clone() else {
            panic!("expected struct");
        };
        let field = item_struct.fields.iter().next().expect("field");
        assert!(has_darling_default(&field.attrs));

        let without: syn::File = syn::parse_quote! {
            struct Args { #[darling(map = "demo_map")] demo: String }
        };
        let Item::Struct(item_struct) = without.items.first().expect("struct").clone() else {
            panic!("expected struct");
        };
        let field = item_struct.fields.iter().next().expect("field");
        assert!(!has_darling_default(&field.attrs));
    }

    #[test]
    fn impl_cache_serves_repeat_resolutions() {
        let root = std::env::temp_dir().join("rsmack-megamac-impl-cache");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("cached.rs"),
            "pub struct Args;",
        )
        .expect("write cached impl file");
        let _guard = with_manifest_dir(&root);
        let env = test_env();
        let args = args_from(quote! { kind = "Func", name = "cached" });
        let src = rsmack_utils::fs::package_src_folder();
        let impls: Ident = syn::parse_quote!(impls);
        let path = impl_file_path(&src, &impls, &args.name);
        let first =
            resolve_analysis(&args, &env, &src, &impls, MacroKind::Func).expect("first resolution");
        let second = resolve_analysis(&args, &env, &src, &impls, MacroKind::Func)
            .expect("second resolution");
        // Both resolutions share the same cached `Arc`, proving the memo served the hit.
        assert!(Arc::ptr_eq(&first, &second));
        let cache = impl_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(cache.contains_key(&path));
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    /// Cached fatals and issues are reproduced on every warm resolution.
    #[test]
    fn impl_cache_replays_fatal_and_issues() {
        let root = std::env::temp_dir().join("rsmack-megamac-cache-replay");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src").join("impls")).expect("create impls dir");
        std::fs::write(
            root.join("src").join("impls").join("replay.rs"),
            "pub struct Args(pub String);",
        )
        .expect("write tuple args impl file");
        let _guard = with_manifest_dir(&root);
        let args = args_from(quote! { kind = "Func", name = "replay" });
        let first = exec(&args, &test_env()).to_string();
        assert!(first.contains("Unsupported `Args` struct"));
        let second = exec(&args, &test_env()).to_string();
        assert_eq!(first, second, "warm resolution must replay the diagnostics");

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }
}
