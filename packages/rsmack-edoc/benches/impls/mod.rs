//! Mirror of `src/impls` for benchmarks: the proc-macro crate type cannot export
//! internals, so benches include the implementation modules directly.
//!
//! Bench builds compile the included files' `#[cfg(test)]` unit tests too (bench
//! targets build with `cfg(test)`), where some macro imports resolve through the bench
//! harness instead of the module imports and read as unused. The shipped lib build
//! requires those imports, so the lint is silenced in this bench-only shim instead of
//! being "fixed" in the real sources.
#![allow(unused_imports)]

#[path = "../../src/impls/const_eval.rs"]
pub mod const_eval;

#[path = "../../src/impls/edoc.rs"]
pub mod edoc;
