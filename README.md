<div align="center">
  <img src="docs/logo.png" alt="rsmack Logo" width="200"/>
  <p>
    <strong>A Rust proc-macro factory: write the <code>exec</code> function, get a documented macro.</strong><br/>
    Multi-crate Cargo workspace where <code>megamac!</code> generates function-like, attribute and derive macros with automatic parameter documentation, prefixed diagnostics built on <code>syn::Error</code>, and zero panics in your expansions. Ship macros like <code>edoc</code> (docs from constants, with const-expression evaluation), <code>wrap</code> (field type wrappers), <code>seanum</code> (SeaORM enums with migrations) and <code>folder_iso_struct</code> (compile-time folder mirroring) — or roll your own in a dozen lines.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.94%2B%20%7C%202024-000000?style=flat-square&logo=rust" alt="Rust"/>
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
2. a `pub fn exec(args, item, env: &ExecEnv) -> TokenStream` that does the actual
   transformation.

Everything else — entrypoint generation for the three macro kinds (single-sourced in the
`rsmack_utils::exec` drivers), documentation extraction, error plumbing, build-time file
generation — is handled by the framework.

> [!TIP]
> Look at [`packages/rsmack-example-macros`](packages/rsmack-example-macros): it is a
> living example crate exercising every macro kind end to end in the test suite.

## Crates

| Crate | Kind | Description |
| --- | --- | --- |
| [`rsmack-megamac`](https://docs.rs/rsmack-megamac) | proc-macro | `megamac!` — generate other proc-macros with automated documentation |
| [`rsmack-utils`](https://docs.rs/rsmack-utils) | library | `ExecEnv`/`Logr` diagnostics, entrypoint drivers, fs helpers |
| [`rsmack-edoc`](https://docs.rs/rsmack-edoc) | proc-macro | `#[edoc]` — struct field docs evaluated from literals, constants and const expressions |
| [`rsmack-wrap`](https://docs.rs/rsmack-wrap) | proc-macro | `#[wrap]` — wrap struct field types, per-field overrides |
| [`rsmack-seanum`](https://docs.rs/rsmack-seanum) | proc-macro | `#[seanum]` — SeaORM active enums with sane defaults + migration helpers |
| [`rsmack-fs`](https://docs.rs/rsmack-fs) | proc-macro | `#[folder_iso_struct]` — mirror a folder structure at compile time, recursively if wanted |

Support tooling (not published): [`rsmack-example-macros`](packages/rsmack-example-macros)
(living examples), [`rsmack-ui-tests`](packages/rsmack-ui-tests) (trybuild snapshots of
every error path), [`benches/`](packages) (criterion expansion benchmarks) and
[`fuzz/`](fuzz) (libfuzzer targets for the parsers).

## Installation

Each macro is a standalone crate — depend only on what you use:

```shell
cargo add rsmack-edoc     # or rsmack-wrap, rsmack-seanum, rsmack-fs
cargo add rsmack-megamac  # to generate your own macros
```

Rust **1.94+** (edition 2024) is required — pinned by `sea-orm 2`.

## Usage

### `edoc` — documentation from constants

Evaluate `#[edoc((...))]` tuples of string literals, constant references and **const
expressions** into real `#[doc]` attributes. The constants file supports literals,
arithmetic/bitwise/shift/comparison expressions, casts, `concat!`, `env!`,
`option_env!` and const-to-const references. Resolution is memoized per file
(process-global, mtime-validated), and every broken expression of a struct is reported
in one pass instead of failing on the first.

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

### `wrap` — wrap field types

Wrap every field of a named struct in a generic type. `Path`, `Slice`, `Tuple` and
`Array` field types are supported; unsupported types keep their original type and emit a
collected error. Per-field `#[wrap(...)]` attributes give fine-grained control.

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
    derives("Display")
)]
#[derive(Default)]
pub enum SwitchAction {
    #[default]
    #[seanum(rename = "send_email")]
    SendEmail,
    SendSms,
}
```

The enum name is snake_cased for `enum_name`, each variant gets its `string_value`
(overridable per variant with `#[seanum(rename = "...")]`), and the derive list
(`Clone`, `Dummy`, `Debug`, `EnumIter`, `DeriveActiveEnum`, `Eq`, `Serialize`,
`Deserialize`, `Hash`, plus any `derives(...)`) is generated for you.

**Integer primary keys**: with an integer `rs_type` (`"i32"`, `"u8"`, ...) the macro
adds `#[repr(...)]`, per-variant `#[sea_orm(num_value = N)]` and an index-based
`TryFrom<rs_type>` implementation. **Migrations**: `migration = true` generates a
`SwitchAction::migration_column_def()` helper (requires the
[`sea-orm-migration`](https://docs.rs/sea-orm-migration) dependency) returning a ready
`ColumnDef` for the enum's column.

### `folder_iso_struct` — compile-time folder mirroring

Annotate a struct, call `YourStruct::generate()` from a build step, and a struct file
mirroring a folder of Rust modules is written into `OUT_DIR`. With
`recursive = true`, subdirectories become nested modules with their own mirrored
structs; `vis` controls the visibility of every generated item.

```rust,ignore
use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(from_crate = "my_crate", folder = "levels", vis = "pub(crate)", recursive = true)]
struct Levels {
    alpha: (),
}

// in build.rs (or anywhere at build time): Levels::generate();
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

Optional knobs: `impls = "my_impls"` / `args = "MyArgs"` rename the implementation
module and args struct, and `doc = "readme" | "inline"` chooses between embedding the
calling crate's `README.md` and generating only the parameter docs (default: `inline`,
**no README dependency**).

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
  [`docs/ERROR_STYLE.md`](docs/ERROR_STYLE.md) for the message style guide and
  [`packages/rsmack-ui-tests`](packages/rsmack-ui-tests) for the rustc-output snapshots.

There is no unwinding, no panic catching, and the generated code carries no
`#[proc_macro_error]` attribute. The parsers are fuzzed (`fuzz/`) to keep it that way.

## Performance

- **Memoized constants**: `edoc` reads and evaluates each constants file once per
  compilation (process-global cache validated by `(mtime, len)`), so N annotated
  structs cost one parse — measured in [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md).
- **Feature-gated `syn`**: `rsmack-utils` compiles without syn's `full` feature; only
  crates that parse full items opt in.
- **Faster iteration for macro authors**: proc-macro recompiles get noticeably quicker
  when dependencies are pre-optimized. Drop this in your `Cargo.toml`:

  ```toml
  [profile.dev.package."*"]
  opt-level = 3
  ```

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
  [`CHANGELOG.md`](CHANGELOG.md).
- **Dependency updates**: [`renovate`](renovate.json) keeps the manifests fresh.

The remaining roadmap (crate split, deeper const-eval, ...) is maintained in
[`docs/IMPROVEMENTS.md`](docs/IMPROVEMENTS.md).
