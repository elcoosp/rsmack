//! Fuzz the darling meta parsing of `megamac::Args`.

#![no_main]

use libfuzzer_sys::fuzz_target;

// The included implementation's expansion entrypoints are unreachable from the fuzz
// harness (only `Args::from_list` is fuzzed), hence the scoped lint exception here only.
#[allow(dead_code, unused_imports)]
#[path = "../../packages/rsmack-megamac/src/impls/megamac.rs"]
mod megamac_impls;

use darling::FromMeta;

fuzz_target!(|data: &[u8]| {
    let tokens = match String::from_utf8_lossy(data).parse::<proc_macro2::TokenStream>() {
        Ok(tokens) => tokens,
        Err(_) => return,
    };
    if let Ok(meta_list) = darling::ast::NestedMeta::parse_meta_list(tokens) {
        let _ = megamac_impls::Args::from_list(&meta_list);
    }
});
