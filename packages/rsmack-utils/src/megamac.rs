//! [rsmack-megamac](../rsmack_megamac/index.html) related utils

use bon::Builder;
use proc_macro2::TokenStream;

use crate::logr::Logr;

#[derive(Debug, Builder)]
#[builder(on(String, into))]
/// Execution environment handed to every macro implementation.
///
/// It carries the diagnostic prefix (usually `concat!(module_path!(), "::", "macro_name")`)
/// and the [`Logr`] collector configured with it.
pub struct ExecEnv {
    /// Prefix prepended to every diagnostic message (e.g. `rsmack_edoc::edoc`).
    #[builder(start_fn)]
    pub prefix: String,
    /// Diagnostics collector pre-configured with [`ExecEnv::prefix`].
    #[builder(field = Logr::builder().prefix(prefix.clone()).build())]
    pub logr: Logr,
}

impl ExecEnv {
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
    fn builder_sets_prefix_and_logr() {
        let env = ExecEnv::builder("my::module::demo").build();
        assert_eq!(env.prefix, "my::module::demo");
        assert_eq!(env.logr.prefix, "my::module::demo");
    }

    #[test]
    fn finish_result_renders_ok_output() {
        let env = ExecEnv::builder("demo").build();
        let output = env.finish_result(Ok(quote! { struct Demo; }));
        assert_eq!(output.to_string(), "struct Demo ;");
    }

    #[test]
    fn finish_result_renders_compile_error_on_err() {
        let env = ExecEnv::builder("demo").build();
        let result: syn::Result<TokenStream> =
            Err(syn::Error::new(proc_macro2::Span::call_site(), "boom"));
        let output = env.finish_result(result);
        assert!(output.to_string().contains("compile_error"));
        assert!(output.to_string().contains("boom"));
    }
}
