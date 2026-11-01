//! This module exposes [`Logr`], a **proc-macro only** diagnostics collector built on
//! [`syn::Error`].
//!
//! Unlike the removed `proc-macro-error2` dependency, diagnostics never panic or unwind:
//! fatal problems are returned as a [`syn::Error`] (abort style, with `?`) while non-fatal
//! problems are collected with [`Logr::emit_error`] and flushed at the end of the macro
//! expansion by [`Logr::finish`].

use std::{cell::RefCell, fmt::Display};

use proc_macro2::{Span, TokenStream};

#[derive(Debug, bon::Builder)]
#[builder(on(String, into))]
/// Diagnostics collector around [`syn::Error`], **only usable inside proc-macro expansions**.
pub struct Logr {
    /// Prefix prepended to every message, usually the attribute path (e.g. `#[edoc]`).
    pub prefix: String,
    /// Collected non-fatal errors, flushed by [`Logr::finish`].
    #[builder(default)]
    errors: RefCell<Vec<syn::Error>>,
}

impl Logr {
    fn fmt_msg<M: Display>(&self, msg: M) -> String {
        format!("#[{}] {}", self.prefix, msg)
    }

    /// Build a prefixed [`syn::Error`] located at `span`.
    #[must_use = "the built error is dropped instead of being returned or emitted"]
    pub fn error<M: Display>(&self, span: Span, msg: M) -> syn::Error {
        syn::Error::new(span, self.fmt_msg(msg))
    }

    /// Build a prefixed [`syn::Error`] located at the macro call site.
    #[must_use = "the built error is dropped instead of being returned or emitted"]
    pub fn call_site_error<M: Display>(&self, msg: M) -> syn::Error {
        self.error(Span::call_site(), msg)
    }

    /// Whether at least one non-fatal error has been collected.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    /// Collect a spanned error and let the macro expansion continue.
    pub fn emit_error<M: Display>(&self, span: Span, msg: M) {
        self.errors.borrow_mut().push(self.error(span, msg));
    }

    /// Append every collected error to `output` and return the merged token stream.
    pub fn finish(&self, mut output: TokenStream) -> TokenStream {
        let mut errors = std::mem::take(&mut *self.errors.borrow_mut());
        if let Some(first) = errors.pop() {
            let mut combined = first;
            for error in errors {
                combined.combine(error);
            }
            output.extend(combined.to_compile_error());
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn fmt_msg_prefixes_message() {
        let logr = Logr::builder().prefix("edoc").build();
        assert_eq!(logr.fmt_msg("oops"), "#[edoc] oops");
    }

    #[test]
    fn error_carries_prefix_and_message() {
        let logr = Logr::builder().prefix("wrap").build();
        let err = logr.error(Span::call_site(), "bad field");
        assert_eq!(err.to_string(), "#[wrap] bad field");
    }

    #[test]
    fn call_site_error_is_prefixed() {
        let logr = Logr::builder().prefix("seanum").build();
        assert_eq!(
            logr.call_site_error("bad args").to_string(),
            "#[seanum] bad args"
        );
    }

    #[test]
    fn finish_appends_collected_errors() {
        let logr = Logr::builder().prefix("edoc").build();
        logr.emit_error(Span::call_site(), "first");
        logr.emit_error(Span::call_site(), "second");
        let rendered = logr.finish(quote! { struct Demo; }).to_string();
        assert!(rendered.contains("compile_error"));
        assert!(rendered.contains("#[edoc] first"));
        assert!(rendered.contains("#[edoc] second"));
        assert!(rendered.contains("struct Demo ;"));
    }

    #[test]
    fn has_errors_tracks_emitted_errors() {
        let logr = Logr::builder().prefix("edoc").build();
        assert!(!logr.has_errors());
        logr.emit_error(Span::call_site(), "oops");
        assert!(logr.has_errors());
        let _ = logr.finish(quote! {});
        assert!(!logr.has_errors());
    }

    #[test]
    fn finish_without_errors_is_identity() {
        let logr = Logr::builder().prefix("edoc").build();
        let output = logr.finish(quote! { struct Demo; });
        assert_eq!(output.to_string(), "struct Demo ;");
        assert_eq!(logr.finish(quote! {}).to_string(), "");
    }
}
