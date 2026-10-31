//! Living examples of macros generated with [`rsmack_megamac::megamac`].
//!
//! This crate is not published: it exists to exercise every macro kind end to end
//! (function-like, attribute and derive) in the workspace test suite, and to serve as a
//! reference implementation for authors of new rsmack-based macros.
//!
//! The generated macros resolve `crate::impls::<name>::exec`, exactly like any
//! rsmack-based proc-macro crate.

mod impls;

use rsmack_megamac::megamac;

megamac!(kind = Func, name = greeting);

megamac!(kind = Attr, name = make_public, receiver = ItemStruct);

megamac!(kind = Derive, name = type_name);
