<div align="center">
  <img src="https://cdn.jsdelivr.net/gh/elcoosp/rsmack@main/docs/logo.png" alt="rsmack Logo" width="200"/>
  <p>
    <strong>The Rust proc-macro factory: write the <code>exec</code> function, get a documented macro.</strong><br/>
    One <code>cargo add rsmack</code> for every macro: <code>edoc</code> (docs from constants, with typed
    const-expression evaluation), <code>wrap</code> (field type wrappers, any type),
    <code>seanum</code> (<code>SeaORM</code> enums with discriminants + migrations),
    <code>folder_iso_struct</code> (inline compile-time folder mirroring) — or roll your own in a
    dozen lines with <code>megamac</code>, the signature-driven macro factory. Prefixed
    diagnostics on <code>syn::Error</code>, zero panics, process-global caches.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.94%2B%20%7C%202024-000000?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License MIT"/>
    <img src="https://img.shields.io/badge/Crates-7-6F4E37?style=flat-square" alt="Crates"/>
    <img src="https://img.shields.io/badge/Coverage-100%25-brightgreen?style=flat-square" alt="Coverage"/>
    <img src="https://img.shields.io/badge/CI-passing-brightgreen?style=flat-square&logo=githubactions" alt="CI"/>
    <img src="https://img.shields.io/badge/Diagnostics-syn%3A%3AError%20%2B%20collected-007ACC?style=flat-square" alt="Diagnostics"/>
    <img src="https://img.shields.io/badge/Changelog-git--cliff-8B0000?style=flat-square" alt="Changelog"/>
  </p>
</div>

---

## Why rsmack?

Writing a proc macro means re-implementing the same boilerplate every time: parse the
attribute tokens, parse the item, map the errors, document the arguments. **rsmack moves
that boilerplate into a workspace of composable crates** so a macro author only writes:

1. a `pub struct Args` with darling-derivable fields — *its doc comments become the
   generated macro's `# Parameters` section*;
2. a `pub fn exec(args, item, env: &ExecEnv) -> TokenStream` that does the actual
   transformation — *or, for DSL macros, an `exec` taking the raw `TokenStream`*.

Everything else — entrypoint generation for the three macro kinds, payload-protocol
selection, documentation extraction, error plumbing, file codegen — is handled by the
framework.

> [!TIP]
> Look at [`packages/rsmack-example-macros`](https://github.com/elcoosp/rsmack/tree/main/packages/rsmack-example-macros): it is a
> living example crate exercising every macro kind (including a raw-payload DSL macro)
> end to end in the test suite.

## Installation

One dependency for everything:

```shell
cargo add rsmack                       # every macro
cargo add rsmack --no-default-features --features edoc   # just one
```

Each macro is also a standalone crate — depend only on what you use:

```shell
cargo add rsmack-edoc     # or rsmack-wrap, rsmack-seanum, rsmack-fs
cargo add rsmack-megamac  # to generate your own macros
```

Rust **1.94+** (edition 2024) is required — pinned by `sea-orm 2`.

## Usage

### `edoc` — documentation from constants

Evaluate `#[edoc((...))]` tuples of string literals, constant references and **const
expressions** into real `#[doc]` attributes — on struct fields, enum variants and the
item itself. The constants file supports literals, arithmetic/bitwise/shift/comparison
expressions, casts, `concat!`, `env!`, `option_env!` and const-to-const references
resolved against **full module paths**. Resolution is memoized per file (process-global,
mtime-validated), and every broken expression of an item is reported in one pass.

```rust
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
```

### `wrap` — wrap field types (any type)

Wrap every field of a named struct in a generic type — paths, slices, tuples, arrays,
references, pointers, function pointers and trait objects all wrap. Per-field
`#[wrap(...)]` attributes give fine-grained control.

```rust
use rsmack_wrap::wrap;

#[wrap(with = Option)]
struct User {
    name: String,                  // becomes Option<String>
    tags: Vec<String>,             // becomes Option<Vec<String>>
    #[wrap(skip)]
    untouched: &'static str,       // stays &'static str
    #[wrap(with = "Box")]
    boxed: String,                 // becomes Box<String>
    #[wrap(with = "Vec", map = "u8")]
    raw: String,                   // becomes Vec<u8> (map replaces the inner type)
}
```

### `seanum` — SeaORM active enums

```rust
use rsmack_seanum::seanum;

#[seanum(
    rs_type = "String",
    db_type = "String(sea_orm::sea_query::StringLen::None)",
    rename_all = "snake_case",
    derives("::serde::Serialize", "::serde::Deserialize")
)]
#[derive(Default)]
pub enum SwitchAction {
    #[default]
    #[seanum(rename = "send_email")]
    SendEmail,
    SendSms,
}
```

Minimal default derives (`Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `EnumIter`,
`DeriveActiveEnum`) — `serde`/`fake` are **opt-in**, never hidden dependencies.
`rename_all` normalizes every non-renamed value, `enum_name = "..."` overrides the table
name. Integer `rs_type` adds `#[repr(...)]`, `#[sea_orm(num_value = N)]` and a
value-based `TryFrom` honoring explicit discriminants (`A = 10, B` continues from 11;
duplicates are reported). `migration = true` generates a `migration_column_def()`
helper (requires [`sea-orm-migration`](https://docs.rs/sea-orm-migration)).

### `folder_iso_struct` — inline compile-time folder mirroring

One attribute, zero build script: the folder is scanned at expansion time and the
mirrored struct is emitted inline. With `recursive = true`, subdirectories become nested
modules with their own mirrored structs; the annotated struct's attributes and
visibility carry over, `derives(...)` injects extra derives on every generated struct.

```rust,ignore
use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(folder = "levels", vis = "pub(crate)", recursive = true, derives("Debug"))]
pub struct Levels {
    // The declared fields are a sketch only: the mirror is generated from the folder.
}
```

### Write your own macro with `megamac`

Declare the macro, write the `impls/<name>.rs` implementation, done:

```rust,ignore
// src/lib.rs
use rsmack_megamac::megamac;

mod impls;

// Generates a function-like `greeting!` macro documented from
// src/impls/greeting.rs's `Args` doc comments.
megamac!(kind = Func, name = greeting);

// Also available: kind = Attr and kind = Derive (the generated derive is the
// PascalCase of `name`). The parsed item type is inferred from the `exec` signature.
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

**The payload protocol is signature-driven** — the `exec` signature *is* the
configuration:

- `exec(args: Args, env: &ExecEnv)` — darling meta parsing, docs extracted from `Args`
  (field docs → `# Parameters`, struct docs → intro, `Option<T>` / `#[darling(default)]`
  fields marked *optional*);
- `exec(input: TokenStream, env: &ExecEnv)` — **raw payload**, DSL macros in ten lines:

```rust,ignore
// src/impls/shout.rs — megamac!(kind = Func, name = shout);
pub fn exec(input: TokenStream, env: &ExecEnv) -> TokenStream {
    let shouted = input.to_string().to_uppercase();
    env.finish(quote::quote! { pub const SHOUTED: &str = #shouted; })
}
```

Optional knobs: `impls = "my_impls"` / `args = "MyArgs"` rename the implementation
module and args struct, and `doc = "readme" | "inline" | "none"` chooses between
embedding the calling crate's `README.md`, generating only the parameter docs (default:
`inline`, no README dependency) or no docs at all. The impl file is parsed once per
process (memoized by `(path, mtime, len)`).

> [!TIP]
> **Playground**: print the expansion a `megamac!` call would emit, without compiling
> anything:
>
> ```shell
> cargo run -p rsmack-megamac --bin playground -- --kind Attr --name edoc
> ```

## Diagnostics without panic

`rsmack-utils::logr::Logr` replaces the abandoned `proc-macro-error2` with a
`syn::Error`-based collector:

- **fatal** problems are returned as a `syn::Error` with `?` and rendered through
  `ExecEnv::finish_result(...)`;
- **non-fatal** problems are collected with `Logr::emit_error` and flushed at the end of
  the expansion, so one bad field does not hide the others;
- every message is prefixed `#[<crate>::<macro>]` — see
  [`docs/ERROR_STYLE.md`](https://github.com/elcoosp/rsmack/blob/main/docs/ERROR_STYLE.md) for the message style guide and
  [`packages/rsmack-ui-tests`](https://github.com/elcoosp/rsmack/tree/main/packages/rsmack-ui-tests) for the rustc-output snapshots.

There is no unwinding, no panic catching, and the generated code carries no
`#[proc_macro_error]` attribute. The parsers are fuzzed (`fuzz/`) to keep it that way.

## Performance

- **Typed const evaluation**: `edoc` keeps constant values machine-typed (`i128`, `f64`,
  ...) until doc assembly — no per-operation string round-trips.
- **Process-global caches**: `edoc` memoizes constants-file resolution per
  `(path, mtime, len)` so N annotated items cost one parse; `megamac` memoizes the
  impl-file analysis so rust-analyzer re-expansions pay a stat, not a parse.
- **Feature-gated `syn`**: `rsmack-utils` compiles without syn's `full` feature; only
  crates that parse full items opt in.
- **Faster iteration for macro authors**: proc-macro recompiles get noticeably quicker
  when dependencies are pre-optimized. Drop this in your `Cargo.toml`:

  ```toml
  [profile.dev.package."*"]
  opt-level = 3
  ```

Measured numbers live in [`docs/BENCHMARKS.md`](https://github.com/elcoosp/rsmack/blob/main/docs/BENCHMARKS.md).

## Development

```shell
cargo test --workspace                        # unit + integration + doc tests
cargo test -p rsmack-ui-tests                 # trybuild UI snapshots
cargo clippy --workspace --all-targets        # pedantic lints are denied (zero allows)
cargo llvm-cov --workspace --lib --bins --tests --ignore-filename-regex 'fuzz' --fail-under-lines 100
cargo bench --workspace                       # criterion expansion benchmarks
bacon                                         # watch mode (see bacon.toml)
```

- **Linting**: `clippy::pedantic` is denied at the workspace level and enforced with
  `-D warnings` in CI, alongside `rustfmt`, `cargo doc` and a
  [`typos`](https://github.com/crate-ci/typos) check.
- **Coverage**: the CI coverage job fails below **100% line coverage**
  (`cargo-llvm-cov`), and the report is uploaded to Codecov.
- **UI tests**: every error path is snapshotted with
  [`trybuild`](https://docs.rs/trybuild) so accidental diagnostic changes fail review.
- **MSRV**: `cargo hack check --each-feature --rust-version` guards the declared MSRV.
- **Semver**: `cargo-semver-checks` runs on pull requests.
- **Releases**: pushing a `v*.*.*` tag runs the release workflow: validation →
  [`git-cliff`](https://git-cliff.org) changelog → ordered `crates.io` publishing →
  GitHub release with generated notes. The changelog itself lives in
  `CHANGELOG.md`.
- **Dependency updates**: `renovate` keeps the manifests fresh.

The v2 design record (brainstorm, breaking decisions, perf ledger) lives in
[`docs/DESIGN_v2.md`](https://github.com/elcoosp/rsmack/blob/main/docs/DESIGN_v2.md); the remaining roadmap is maintained in
[`docs/IMPROVEMENTS.md`](https://github.com/elcoosp/rsmack/blob/main/docs/IMPROVEMENTS.md).
