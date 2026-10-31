//! [rsmack-megamac](../rsmack_megamac/index.html) related utils

use bon::Builder;
use proc_macro2::TokenStream;

use crate::logr::Logr;

#[derive(Debug, Builder)]
#[builder(on(String, into))]
pub struct ExecEnv {
    #[builder(start_fn)]
    pub module_path: String,
    #[builder(start_fn)]
    pub implementations_mod_ident: String,
    #[builder(start_fn)]
    pub exec_args_ident: String,
    #[builder(start_fn)]
    pub exec_fn_mod_ident: String,
    #[builder(
        field = Logr::builder()
            .prefix(format!("{module_path}::{exec_fn_mod_ident}"))
            .build()
    )]
    pub logr: Logr,
}

impl ExecEnv {
    /// Flush the [`Logr`] collected errors into `output` and return the merged token stream.
    pub fn finish(&self, output: TokenStream) -> TokenStream {
        self.logr.finish(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_fields_and_logr_prefix() {
        let env = ExecEnv::builder("my::module", "impls", "Args", "demo").build();
        assert_eq!(env.module_path, "my::module");
        assert_eq!(env.implementations_mod_ident, "impls");
        assert_eq!(env.exec_args_ident, "Args");
        assert_eq!(env.exec_fn_mod_ident, "demo");
        assert_eq!(env.logr.prefix, "my::module::demo");
    }
}
