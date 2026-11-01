# rsmack improvements & roadmap

This document collects the outcome of an engineering review of the workspace and an
ambitious brainstorm of where the project could go next. It is organised in three parts:
what the 0.19/0.20 passes already implemented, then remaining improvement tracks for
**performance** and **modularity / DRY / KISS**, plus the future **complementary
features** backlog, each with an impact/effort rating.

## Already implemented in this pass

These items were part of the 0.19.0/0.20.0 overhauls and are listed here for context:

- **No more `proc-macro-error2`** (future-incompatible, abandoned): diagnostics are now
  built on `syn::Error` — fatal errors flow through `?`, non-fatal ones are collected in
  `Logr` and flushed by `ExecEnv::finish`. No unwinding, no panic catching, one less
  dependency.
- **All majors bumped**: syn 2→3, darling 0.21→0.24, sea-orm 1→2, fake 4→5,
  stringcase 0.4→1.0, strum 0.27→0.28, edition 2024, resolver 3.
- **Workspace-level dependency management**: every external crate is declared once in
  `[workspace.dependencies]`; versions, edition, license and rust-version live in
  `[workspace.package]` (DRY).
- **edoc**: the constants file is parsed as a real AST with `syn::visit` instead of
  line-scanning (multi-line consts, nested modules, no more `unimplemented!()` panics on
  non-doc attributes, qualified const paths resolve).
- **seanum**: snake_casing delegated to `stringcase` instead of a hand-rolled loop.
- **fs**: `calling_crate_dir` walks up to the nearest `Cargo.toml` (robust to custom
  target dirs, memoized in a `OnceLock`); `from_crate`/`folder` accept relative paths.
- **Empty attribute/macro arguments map to darling's `from_word`**, so unit-struct `Args`
  work with `#[my_macro]` (no parens) under darling 0.24.
- **Test architecture**: unit tests drive every `exec` with parsed inputs, integration
  tests compile the generated macros (including a dedicated example crate), negative
  paths are covered — 100% line/region/function coverage enforced in CI.

### Implemented from the improvement tracks (0.20.0)

**Performance**

- ✅ **Memoized constants-file resolution**: process-global `OnceLock` cache keyed by
  path and validated by `(mtime, len)` — N structs against the same `consts.rs` cost one
  parse; cached issues are re-emitted so every struct reports them.
- ✅ **Feature-gated `syn`**: the workspace `syn` dependency is
  `default-features = false`; `full`/`visit`/`extra-traits` are opted into per crate,
  and `rsmack-utils` compiles with the light default set only.
- ✅ **Benchmark harness**: criterion benches for every macro implementation
  (`cargo bench --workspace`), results and analysis in
  [`docs/BENCHMARKS.md`](BENCHMARKS.md), plus a `workflow_dispatch` bench workflow.
- ✅ **Avoided `ItemStruct` clones in `edoc`**: the expansion consumes the parsed item in
  place (already the case after the 0.19 pass; re-verified).

**Modularity & DRY & KISS**

- ✅ **Shared exec driver**: `rsmack_utils::exec::{attr,func,derive}_entrypoint`
  single-sources darling meta parsing, item parsing, `from_word` handling and the
  `finish_result` rendering; generated macros call the drivers through closures, which
  also removed the `crate_in_macro_def` allow (the last clippy exemption is gone —
  pedantic is now denied with **zero allows**).
- ✅ **`Logr` as the only error facade**: darling meta errors are routed through `Logr`
  (preserving their spans), so every diagnostic carries the same
  `#[<crate>::<macro>]` prefix.
- ✅ **Arbitrary impl/args idents in `megamac`**: `megamac!(impls = "...", args = "...")`.
- ✅ **Killed the `include_str!("../README.md")` hard dependency**: `doc = "readme" |
  "inline"` (default `inline`); the parsed item type is now inferred from the `exec`
  signature, so the `receiver` option is gone too.
- ✅ **Single error-message style guide**: [`docs/ERROR_STYLE.md`](ERROR_STYLE.md),
  enforced by the trybuild snapshots.
- ✅ **Multi-error `megamac` docs**: documentation problems of the `Args` struct are
  collected instead of panicking on the first (`expect("named field")` is gone).

**Complementary features**

- ✅ **trybuild UI test suite**: `packages/rsmack-ui-tests` snapshots the rustc output of
  every error path (edoc unresolved const / non-tuple / unsupported element / missing
  file, wrap tuple + unsupported type, fs tuple struct, megamac bad kind / missing /
  broken impl, seanum integer violations).
- ✅ **`edoc` const-expression evaluation**: a mini `min_const_eval`-style interpreter
  for literals, arithmetic/bitwise/shift/comparison ops, casts, `concat!`, `env!`,
  `option_env!` and const-to-const references with cycle detection.
- ✅ **`edoc` multi-error accumulation**: every broken expression of a struct is
  collected and rendered together; failed attributes are dropped instead of leaking an
  unknown-attribute error.
- ✅ **`wrap` skip / rename attributes**: `#[wrap(skip)]`, per-field
  `#[wrap(with = "...")]` and `#[wrap(with = "...", map = "...")]`.
- ✅ **`seanum` custom derives + variant renames + integer `rs_type`**: `derives(...)`,
  `#[seanum(rename = "...")]`, integer `rs_type` with `#[repr]`, `num_value` indices and
  automatic `TryFrom` impls.
- ✅ **`seanum` integration with `sea-orm-migration`**: `migration = true` generates a
  `migration_column_def()` helper.
- ✅ **`folder_iso_struct` recursion + visibility**: `recursive = true` renders
  subdirectories as nested modules with their own structs; `vis = "pub(crate)"` (any
  `syn::Visibility` string) applies to every generated item.
- ✅ **`megamac` playground**: `cargo run -p rsmack-megamac --bin playground` prints the
  expansion of a rsmack-based macro from the command line.
- ✅ **`cargo-semver-checks` in CI**: pull-request job.
- ✅ **MSRV + docs.rs jobs**: `cargo hack check --each-feature --rust-version` on
  rust 1.94 (the honest floor: `sea-orm 2` requires it) and
  `[package.metadata.docs.rs]` all-features on every published crate.
- ✅ **Fuzz the parsers**: `fuzz/` libfuzzer targets for the consts parser
  (`edoc_consts_source`) and every `Args::from_list` meta parser.

## Performance (remaining)

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Cache megamac impl-file parses | Low | Low | The same `impls/<name>.rs` is re-read per generated macro; a `OnceLock` memo like the edoc consts cache would shave ~1 ms per macro (measured in `docs/BENCHMARKS.md`). Marginal because it runs once per macro. |
| Skip doc extraction when `doc = "inline"` and no params | Low | Low | The impl file is still read for `Args`; a "no docs" fast path would skip the read entirely. |

## Modularity & DRY & KISS (remaining)

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Split `rsmack-utils` by concern | Medium | Medium | `exec` (macro boilerplate) and `fs` (build-time codegen) have disjoint users. Two smaller crates (`rsmack-exec`, `rsmack-codegen-fs`) would shrink dependency trees for consumers that only need one side. Deferred: with the syn feature gating in place the dependency delta is small, and the split breaks the `rsmack_utils::` paths embedded in generated code. |

## Complementary features backlog

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Fuzzing in CI (scheduled) | Medium | Medium | Run `cargo fuzz build` + short fuzz runs on a weekly schedule; the targets exist in `fuzz/`, a scheduled job keeps regressions from landing. |
| `edoc` richer const-eval (structs, `include!`) | Low | High | The evaluator covers the documented subset; full rustc const-eval parity would need a real interpreter or `const_eval` crate integration. |
| `seanum` explicit discriminant support | Low | Low | Integer `rs_type` currently requires implicit index values; honoring explicit discriminants needs resolving duplicates and gaps. |
| `folder_iso_struct` derive injection per nested struct | Low | Low | The top-level `pre` attributes apply to every generated struct; per-depth derives would need a dedicated option. |
| `megamac` playground expansion of *macro invocations* (not just entrypoints) | Medium | High | Printing the expansion of `greeting!(who = "x")` requires executing the user's `exec` in a sandbox — only realistic via `cargo expand` integration. |

## Suggested sequencing

1. **Quick wins**: megamac impl-file cache, "no docs" fast path.
2. **Structural**: `rsmack-utils` crate split (when the generated-path break is
   acceptable), scheduled fuzzing job.
3. **Ambitious**: full const-eval parity, invocation-level playground.
