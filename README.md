<div align="center">
  <img src="docs/logo.png" alt="rsmack Logo" width="200"/>
  <p>
    <strong>A Rust proc-macro factory: write the <code>exec</code> function, get a documented macro.</strong><br/>
    Multi-crate Cargo workspace where <code>megamac!</code> generates function-like, attribute and derive macros with automatic parameter documentation, prefixed diagnostics built on <code>syn::Error</code>, and zero panics in your expansions. Ship macros like <code>edoc</code> (docs from constants), <code>wrap</code> (field type wrappers), <code>seanum</code> (SeaORM enums) and <code>folder_iso_struct</code> (compile-time folder mirroring) — or roll your own in a dozen lines.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.85%2B%20%7C%202024-000000?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License MIT"/>
    <img src="https://img.shields.io/badge/Crates-6-6F4E37?style=flat-square" alt="Crates"/>
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
2. a `pub fn exec(args, item, env) -> TokenStream` that does the actual transformation.

Everything else — entrypoint generation for the three macro kinds, documentation
extraction, error plumbing, build-time file generation — is handled by the framework.

> [!TIP]
> Look at [`packages/rsmack-example-macros`](packages/rsmack-example-macros): it is a
> living example crate exercising every macro kind end to end in the test suite.

## Crates

| Crate | Kind | Description |
| --- | --- | --- |
| [`rsmack-megamac`](https://docs.rs/rsmack-megamac) | proc-macro | `megamac!` — generate other proc-macros with automated documentation |
| [`rsmack-utils`](https://docs.rs/rsmack-utils) | library | `call_*_proc_macro` boilerplate, `ExecEnv`/`Logr` diagnostics, fs helpers |
| [`rsmack-edoc`](https://docs.rs/rsmack-edoc) | proc-macro | `#[edoc]` — struct field docs evaluated from string literals + constants |
| [`rsmack-wrap`](https://docs.rs/rsmack-wrap) | proc-macro | `#[wrap]` — wrap every struct field type in a generic type |
| [`rsmack-seanum`](https://docs.rs/rsmack-seanum) | proc-macro | `#[seanum]` — SeaORM active enums with sane defaults |
| [`rsmack-fs`](https://docs.rs/rsmack-fs) | proc-macro | `#[folder_iso_struct]` — mirror a folder structure at compile time |

## Installation

Each macro is a standalone crate — depend only on what you use:

```shell
cargo add rsmack-edoc     # or rsmack-wrap, rsmack-seanum, rsmack-fs
cargo add rsmack-megamac  # to generate your own macros
```

Rust **1.85+** (edition 2024) is required.

## Usage

### `edoc` — documentation from constants

Evaluate `#[edoc((...))]` tuples of string literals and constants into real `#[doc]`
attributes. Unresolved constants are reported without aborting the other fields.

```rust
use rsmack_edoc::edoc;

mod consts {
    pub const APP: &str = "myapp";
    pub const VERSION: &str = "1.2.3";
}

#[edoc(from = self::consts)]
struct Client {
    #[edoc(("Client ", APP, " v", VERSION))]
    name: String,
}
```

### `wrap` — wrap field types

Wrap every field of a named struct in a generic type. `Path`, `Slice`, `Tuple` and
`Array` field types are supported; unsupported types keep their original type and emit a
collected error.

```rust
use rsmack_wrap::wrap;

#[wrap(with = Option)]
struct User {
    name: String,        // becomes Option<String>
    tags: Vec<String>,   // becomes Option<Vec<String>>
}
```

### `seanum` — SeaORM active enums

```rust
use rsmack_seanum::seanum;

#[seanum(rs_type = "String", db_type = "Enum")]
#[derive(Default)]
pub enum SwitchAction {
    #[default]
    SendEmail,
    SendSms,
}
```

The enum name is snake_cased for `enum_name`, each variant gets its `string_value`, and
the derive list (`Clone`, `Dummy`, `Debug`, `EnumIter`, `DeriveActiveEnum`, `Eq`,
`Serialize`, `Deserialize`, `Hash`) is generated for you.

> [!NOTE]
> With SeaORM 2, string columns need an explicit length:
> `db_type = "String(sea_orm::sea_query::StringLen::None)"`.

### `folder_iso_struct` — compile-time folder mirroring

Annotate a struct, call `YourStruct::generate()` from a build step, and a struct file
mirroring a flat folder of Rust modules is written into `OUT_DIR`.

```rust,ignore
use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(from_crate = "my_crate", folder = "levels")]
struct Levels {
    alpha: (),
}

// in build.rs (or anywhere at build time): Levels::generate();
// writes $OUT_DIR/Levels.rs with `alpha: crate::levels::alpha::Alpha` fields
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

// Also available: kind = Attr (with receiver = ItemStruct, ItemEnum, ...) and
// kind = Derive (the generated derive is the PascalCase of `name`).
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

pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    let who = args.who;
    env.finish(quote::quote! {
        pub const GREETING: &str = concat!("Hello ", #who, "!");
    })
}
```

> [!WARNING]
> `megamac` reads `impls/<name>.rs` from the calling crate at expansion time and embeds
> `include_str!("../README.md")` in the generated macro docs, so the macro-defining
> crate needs a `README.md` file.

## Diagnostics without panic

`rsmack-utils::logr::Logr` replaces the abandoned `proc-macro-error2` with a
`syn::Error`-based collector:

- **fatal** problems are returned as a `syn::Error` with `?` and rendered through
  `env.finish(...)`;
- **non-fatal** problems are collected with `Logr::emit_error` and flushed at the end of
  the expansion, so one bad field does not hide the others.

There is no unwinding, no panic catching, and the generated code carries no
`#[proc_macro_error]` attribute.

## Development

```shell
cargo test --workspace                        # unit + integration + doc tests
cargo clippy --workspace --all-targets        # pedantic lints are denied
cargo llvm-cov --workspace --all-targets --fail-under-lines 100
bacon                                         # watch mode (see bacon.toml)
```

- **Linting**: `clippy::pedantic` is denied at the workspace level and enforced with
  `-D warnings` in CI, alongside `rustfmt`, `cargo doc` and a
  [`typos`](https://github.com/crate-ci/typos) check.
- **Coverage**: the CI coverage job fails below **100% line coverage**
  (`cargo-llvm-cov`), and the report is uploaded to Codecov.
- **Releases**: pushing a `v*.*.*` tag runs the release workflow: validation →
  [`git-cliff`](https://git-cliff.org) changelog → ordered `crates.io` publishing →
  GitHub release with generated notes. The changelog itself lives in
  [`CHANGELOG.md`](CHANGELOG.md).
- **Dependency updates**: [`renovate`](renovate.json) keeps the manifests fresh.

An ambitious list of follow-up ideas (performance, modularity, DRY, new features) is
maintained in [`docs/IMPROVEMENTS.md`](docs/IMPROVEMENTS.md).
