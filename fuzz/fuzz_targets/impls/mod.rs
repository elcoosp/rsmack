//! Mirror of the rsmack-edoc `src/impls` for fuzzing (proc-macro crates cannot export
//! plain items, so the implementation modules are included directly).

#[path = "../../../packages/rsmack-edoc/src/impls/const_eval.rs"]
pub mod const_eval;

#[path = "../../../packages/rsmack-edoc/src/impls/edoc.rs"]
pub mod edoc;
