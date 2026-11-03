//! Function-like **raw payload** macro: the signature-driven v2 protocol in action.
//!
//! `exec` takes a `proc_macro2::TokenStream` first parameter, so `megamac` generates a
//! `func_raw_entrypoint` entrypoint: the payload tokens are forwarded untouched — the
//! DSL-macro pattern (`sql!`, `html!`, ...) in ten lines.

use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

// The raw entrypoint driver hands the payload over by value (`FnOnce` contract),
// stringifying it is enough for this example.
#[expect(clippy::needless_pass_by_value)]
pub fn exec(input: TokenStream, env: &ExecEnv) -> TokenStream {
    // The payload words are echoed upper-cased into a constant.
    let shouted = input.to_string().to_uppercase();
    env.finish(quote::quote! {
        pub const SHOUTED: &str = #shouted;
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_is_upper_cased() {
        let env = ExecEnv::new("test::shout");
        let output = exec(syn::parse_quote! { hello world }, &env).to_string();
        assert!(output.contains("HELLO WORLD"));
        assert!(!output.contains("compile_error"));
    }
}
