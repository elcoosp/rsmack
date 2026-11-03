# rsmack

The Rust proc-macro factory, behind one facade: every rsmack macro with a single
dependency.

```shell
cargo add rsmack                       # everything
cargo add rsmack --no-default-features --features edoc   # just one macro
```

| Feature | Macro | Purpose |
| --- | --- | --- |
| `edoc` | `#[edoc]` | docs from constants, with const-expression evaluation |
| `wrap` | `#[wrap]` | wrap struct field types, per-field overrides |
| `seanum` | `#[seanum]` | `SeaORM` active enums with sane defaults + migrations |
| `fs` | `#[folder_iso_struct]` | compile-time folder mirroring, inline |
| `megamac` | `megamac!` | generate your own documented macros from an `exec` fn |

Individual crates (`rsmack-edoc`, `rsmack-wrap`, ...) stay published for precise
dependency control. Authoring macros with `megamac` additionally requires a direct
[`rsmack-utils`](https://docs.rs/rsmack-utils) dependency (proc-macro crates cannot be
imported as normal modules).

See the [workspace README](https://github.com/elcoosp/rsmack) for the full
documentation.
