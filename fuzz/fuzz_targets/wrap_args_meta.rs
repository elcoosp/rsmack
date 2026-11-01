//! Fuzz the darling meta parsing of `wrap::Args` and its per-field config.

#![no_main]

use libfuzzer_sys::fuzz_target;

// The included implementation's expansion entrypoints are unreachable from the fuzz
// harness (only `Args::from_list` is fuzzed), hence the scoped lint exception here only.
#[allow(dead_code, unused_imports)]
#[path = "../../packages/rsmack-wrap/src/impls/wrap.rs"]
mod wrap;

use darling::FromMeta;

fuzz_target!(|data: &[u8]| {
    let tokens = match String::from_utf8_lossy(data).parse::<proc_macro2::TokenStream>() {
        Ok(tokens) => tokens,
        Err(_) => return,
    };
    if let Ok(meta_list) = darling::ast::NestedMeta::parse_meta_list(tokens) {
        let _ = wrap::Args::from_list(&meta_list);
    }
});
