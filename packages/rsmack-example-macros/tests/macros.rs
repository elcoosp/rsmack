//! End-to-end tests invoking every kind of megamac-generated macro.

use rsmack_example_macros::{TypeName, make_public};

// Function-like macro generated with `megamac!(kind = Func, ...)`
rsmack_example_macros::greeting!(who = "integration tests");

#[test]
fn func_macro_expands() {
    assert_eq!(GREETING, "Hello integration tests!");
}

// Attribute macro generated with `megamac!(kind = Attr, ...)`
#[make_public]
struct Demo {
    a: u8,
}

#[test]
fn attr_macro_expands() {
    let demo = Demo { a: 7 };
    assert_eq!(demo.a, 7);
}

// Derive macro generated with `megamac!(kind = Derive, ...)`
trait TypeName {
    fn type_name() -> &'static str;
}

#[derive(TypeName)]
struct User;

#[test]
fn derive_macro_expands() {
    assert_eq!(User::type_name(), "User");
}
