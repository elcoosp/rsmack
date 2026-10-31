# rsmack-example-macros

Living examples of macros generated with `rsmack_megamac::megamac` (function-like,
attribute and derive). This crate is not published: it exists to exercise every macro kind
end to end in the workspace test suite and to serve as a reference for authors of new
rsmack-based macros.

```rust,ignore
use rsmack_megamac::megamac;

mod impls;

megamac!(kind = Derive, name = type_name);
```

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
