//! [rsmack-megamac](../rsmack_megamac/index.html) related utils

use proc_macro2::TokenStream;

use crate::logr::Logr;

/// Execution environment handed to every macro implementation.
///
/// It carries the diagnostic prefix (usually `concat!(module_path!(), "::", "macro_name")`)
/// and the [`Logr`] collector configured with it.
pub struct ExecEnv {
    /// Prefix prepended to every diagnostic message (e.g. `rsmack_edoc::edoc`).
    pub prefix: String,
    /// Diagnostics collector pre-configured with [`ExecEnv::prefix`].
    pub logr: Logr,
}

impl ExecEnv {
    /// Build the environment for a macro whose diagnostics are prefixed with `prefix`.
    #[must_use]
    pub fn new(prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        let logr = Logr::new(prefix.clone());
        Self { prefix, logr }
    }

    /// Flush the [`Logr`] collected errors into `output` and return the merged token stream.
    pub fn finish(&self, output: TokenStream) -> TokenStream {
        self.logr.finish(output)
    }

    /// The shared exec driver: render `result` when successful, otherwise append its
    /// `compile_error!` tokens. Every `match run(...) { ... }` boilerplate collapses to
    /// `env.finish_result(run(args, item, &env))`.
    pub fn finish_result(&self, result: syn::Result<TokenStream>) -> TokenStream {
        let output = result.unwrap_or_else(|error| error.to_compile_error());
        self.finish(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn new_sets_prefix_and_logr() {
        let env = ExecEnv::new("my::module::demo");
        assert_eq!(env.prefix, "my::module::demo");
        assert_eq!(env.logr.prefix, "my::module::demo");
    }

    #[test]
    fn new_accepts_string_and_str() {
        let owned = String::from("owned::prefix");
        assert_eq!(ExecEnv::new(owned).prefix, "owned::prefix");
        assert_eq!(ExecEnv::new("borrowed").prefix, "borrowed");
    }

    #[test]
    fn finish_result_renders_ok_output() {
        let env = ExecEnv::new("demo");
        let output = env.finish_result(Ok(quote! { struct Demo; }));
        assert_eq!(output.to_string(), "struct Demo ;");
    }

    #[test]
    fn finish_result_renders_compile_error_on_err() {
        let env = ExecEnv::new("demo");
        let result: syn::Result<TokenStream> =
            Err(syn::Error::new(proc_macro2::Span::call_site(), "boom"));
        let output = env.finish_result(result);
        assert!(output.to_string().contains("compile_error"));
        assert!(output.to_string().contains("boom"));
    }
}
