//! Fuzz the darling meta parsing of `edoc::Args`.

#![no_main]

use libfuzzer_sys::fuzz_target;

// The proc-macro crate type cannot export internals, so the implementation modules are
// included directly. The included `#[cfg(test)]` unit tests and the non-parsing parts
// are unreachable from the fuzz harness, hence the scoped lint exceptions here only.
#[allow(dead_code, unused_imports)]
mod impls;

use darling::FromMeta;

fuzz_target!(|data: &[u8]| {
    let tokens = match String::from_utf8_lossy(data).parse::<proc_macro2::TokenStream>() {
        Ok(tokens) => tokens,
        Err(_) => return,
    };
    if let Ok(meta_list) = darling::ast::NestedMeta::parse_meta_list(tokens) {
        let _ = impls::edoc::Args::from_list(&meta_list);
    }
});
