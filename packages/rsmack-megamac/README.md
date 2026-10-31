# rsmack-megamac

Procedural macro generator with automatic documentation and error handling.

`megamac!` reads the `impls/<name>.rs` file of the calling crate, extracts the documented
`Args` struct of the implementation and generates the proc-macro entrypoint (function-like,
attribute or derive) prefixed with a `# Parameters` section built from the `Args`
documentation.

```rust,ignore
use rsmack_megamac::megamac;

megamac!(kind = Attr, name = wrap, receiver = ItemStruct);
```

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation,
and [`rsmack-example-macros`](../rsmack-example-macros) for living examples of every macro
kind.
