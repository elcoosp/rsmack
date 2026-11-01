//! Fuzz the edoc constants-file parser and mini const-evaluator.
//!
//! Arbitrary input must never panic: invalid rust fails to parse, valid rust either
//! resolves constants or reports non-fatal issues.

#![no_main]

use libfuzzer_sys::fuzz_target;

// The proc-macro crate type cannot export internals, so the implementation modules are
// included directly, mirroring the bench harness layout.
// The proc-macro crate type cannot export internals, so the implementation modules are
// included directly. The included `#[cfg(test)]` unit tests and the non-parsing parts
// are unreachable from the fuzz harness, hence the scoped lint exceptions here only.
#[allow(dead_code, unused_imports)]
mod impls;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    // Errors are values here: parse failures and evaluation issues must not panic.
    let _ = impls::edoc::parse_consts_source(&source);
});
