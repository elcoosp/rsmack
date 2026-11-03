# rsmack-edoc

`#[edoc]` — documentation from constants, with const-expression evaluation.

Evaluate `#[edoc((...))]` tuples of string literals, constant references and **const
expressions** into real `#[doc]` attributes on struct fields, enum variants and the
annotated item itself.

```rust,ignore
use rsmack_edoc::edoc;

mod consts {
    pub const APP: &str = "myapp";
    pub const MAJOR: i32 = 1;
    pub const TAG: &str = concat!("v", MAJOR);
}

#[edoc(from = self::consts)]
struct Client {
    #[edoc(("Client ", APP, " tag ", TAG, " build ", 2 * 21))]
    name: String,
}

#[edoc(from = self::consts, ("The ", APP, " client states"))]
enum State {
    #[edoc(("Idle since ", TAG))]
    Idle,
    Busy,
}
```

Highlights:

- **Typed const evaluation**: literals, arithmetic/bitwise/shift/comparison operations,
  casts (`as`), `concat!`, `env!`, `option_env!` — values stay machine-typed until doc
  assembly, no string round-trips.
- **Path-aware constants**: constants resolve against their full module path
  (`nested::DEEP`); a bare name resolves when unique, and ambiguity is reported with
  every candidate.
- **Memoized per file**: the constants file is parsed and evaluated once per compilation
  (process-global cache validated by `(mtime, len)`); N annotated items cost one parse.
- **Multi-error accumulation**: every broken expression of an item is reported in one
  pass instead of failing on the first.

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
