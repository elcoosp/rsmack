# rsmack-megamac

Procedural macro generator with automatic documentation and error handling.

Declare the macro, write the `impls/<name>.rs` implementation, done:

```rust,ignore
// src/lib.rs of your proc-macro crate
use rsmack_megamac::megamac;

mod impls;

megamac!(kind = Func, name = greeting);
```

```rust,ignore
// src/impls/greeting.rs
use darling::FromMeta;
use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

/// Greeting configuration
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Recipient of the greeting
    pub who: String,
}

pub fn exec(args: Args, env: &ExecEnv) -> TokenStream {
    let who = args.who;
    env.finish(quote::quote! {
        pub const GREETING: &str = concat!("Hello ", #who, "!");
    })
}
```

The **payload protocol is signature-driven**:

- `exec(args: Args, env: &ExecEnv)` — the payload is darling-parsed into the documented
  args struct (its field docs become the generated macro's `# Parameters` section, its
  struct docs become the intro, `Option<T>`/`#[darling(default)]` fields are marked
  optional);
- `exec(input: TokenStream, env: &ExecEnv)` (`Func`/`Attr`) — the raw payload tokens are
  forwarded untouched, unlocking DSL macros (`sql!`, `html!`, ...) in a dozen lines.

`kind = Attr` / `kind = Derive` infer the parsed item type from the `exec` signature;
derives receive no payload and get no `# Parameters` section. Optional knobs: `impls =
"my_impls"` / `args = "MyArgs"` rename the implementation module and args struct, and
`doc = "readme" | "inline" | "none"` chooses the documentation source (default:
`inline`, no README dependency). The impl file is parsed once per process (memoized by
`(path, mtime, len)`).

The parsed item type of `Attr` and `Derive` macros is inferred from the `exec`
signature, so no receiver configuration is needed.

Your proc-macro crate needs both `rsmack-megamac` (build-time generation) and
`rsmack-utils` (the `ExecEnv` type and the generated entrypoint drivers — proc-macro
crates cannot be imported as normal modules).

> **Playground**: print the expansion a `megamac!` call would emit, without compiling
> anything:
>
> ```shell
> cargo run -p rsmack-megamac --bin playground -- --kind Attr --name edoc
> ```

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation,
and [`rsmack-example-macros`](https://github.com/elcoosp/rsmack/tree/main/rsmack-example-macros) for living examples of every
macro kind (including a raw-payload DSL macro).
