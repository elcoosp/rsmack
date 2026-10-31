# rsmack improvements & roadmap

This document collects the outcome of an engineering review of the workspace and an
ambitious brainstorm of where the project could go next. It is organised in four parts:
what this pass already implemented, then improvement tracks for **performance**,
**modularity / DRY / KISS**, and **complementary features**, each with an impact/effort
rating.

## Already implemented in this pass

These items were part of the 0.19.0 overhaul and are listed here for context:

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
- **megamac**: duplicate `FieldDoc` entries fixed, missing-`Args` reported as a proper
  compile error, `Attr` receiver defaults to `ItemStruct`.
- **Empty attribute/macro arguments map to darling's `from_word`**, so unit-struct `Args`
  work with `#[my_macro]` (no parens) under darling 0.24.
- **Test architecture**: unit tests drive every `exec` with parsed inputs, integration
  tests compile the generated macros (including a dedicated example crate), negative
  paths are covered — 100% line/region/function coverage enforced in CI.

## Performance

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Memoize constants-file resolution (`OnceLock<HashMap<PathBuf, HashMap<String, String>>>`) | Medium | Low | The old `TODO` in `edoc.rs`: the same `consts.rs` is re-read and re-parsed for every annotated struct in a crate. A process-global memo keyed by path + mtime makes N structs cost one parse. |
| Avoid `ItemStruct` clones in `edoc` | Low | Low | `replace_edoc_attrs` mutates in place already; the remaining `parse2::<DeriveInput>` round-trip can go away by parsing `edoc` fields directly from `&item.fields` (already the case) and dropping the darling derive-input pass entirely. |
| Feature-gate `syn/features = ["full"]` per crate | Low | Medium | Only `edoc`/`wrap`/`fs` need `full`; `megamac` could compile with `derive`, `parsing`, `printing` only, cutting proc-macro dylib size and compile time. |
| Compile the workspace with `profile.dev.package."*".opt-level = 3` guidance in docs | Low | Low | Proc-macro authors iterate faster when dependencies are optimized; document the recipe instead of forcing it. |
| Benchmark harness for macro expansion time (divan or `cargo expand` + hyperfine) | Medium | Medium | Guard against regressions in the hot path (parsing + quoting). A `benches/expansion.rs` measuring `megamac!` + `edoc` expansions would make performance work measurable. |

## Modularity & DRY & KISS

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Extract a shared `exec` driver | Medium | Medium | Every crate repeats `match run(...) { Ok => env.finish, Err => env.finish(compile_error) }`. A `rsmack_utils::exec::run_with_env(args, item, env, run_fn)` helper (or a small trait `MacroExec`) would make the pattern single-sourced; macro-generated entrypoints could call it directly. |
| Make `Logr` the only error facade | Medium | Low | `rsmack_utils::exec` still uses `darling::Error::write_errors` directly for meta parsing. Routing those through `Logr` would give every error the same `#[macro_name]` prefix. |
| Split `rsmack-utils` by concern | Medium | Medium | `exec` (macro boilerplate) and `fs` (build-time codegen) have disjoint users. Two smaller crates (`rsmack-exec`, `rsmack-codegen-fs`) would shrink dependency trees for consumers that only need one side. |
| Support arbitrary impl/args idents in `megamac` | Medium | Low | `call_*_impls_with_args!` hardcodes `impls` + `Args`. Accepting optional idents (`megamac!(kind = Attr, name = x, impls = attributes, args = XArgs)`) costs two more darling fields. |
| Kill the `include_str!("../README.md")` hard dependency | Medium | Low | Generated macros fail to compile when the consumer crate has no README. Make it a megamac option (`doc = "readme"` / `doc = "inline"`), defaulting to inline generated docs. |
| Single error-message style guide | Low | Low | Messages currently mix `Failed to ...` / `Only ... supported` styles. A tiny doc + clippy rule keeps user-facing errors uniform. |

## Complementary features (be ambitious)

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| **trybuild UI test suite** | High | Medium | Snapshot the exact rustc output of every error path (`edoc` unresolved const, `wrap` unsupported type, tuple-struct aborts...). Pair with the `Logr` prefix system for stable, grep-able diagnostics; catch accidental message changes in review. |
| **`edoc` const-expression evaluation** | High | High | Evaluate `concat!(...)`, arithmetic and `env!` constants with `min_const_eval`-style interpretation (or the `const_eval` crate) instead of only literals — the current limitation is documented and users hit it quickly. |
| **`edoc` multi-error accumulation** | Medium | Low | Today one unresolved const emits one collected error; accumulating *all* unresolved consts per struct and rendering them together is a UX win. |
| **`wrap` skip / rename attributes** | Medium | Low | `#[wrap(skip)]` on fields, `#[wrap(with = Option, map = "...")]` for per-field wrappers — natural extension of the current all-or-nothing model. |
| **`seanum` custom derives + variant renames** | Medium | Low | Accept extra derives (`derives(...)`), per-variant `#[seanum(rename = "...")]`, and integer `rs_type` with automatic `TryFrom` impls. |
| **`seanum` integration with `sea-orm-migration`** | Medium | Medium | Generate the matching migration helper (`ColumnDef` insertion) next to the enum, removing boilerplate from migration files. |
| **`folder_iso_struct` recursion + visibility** | Medium | Medium | Support nested folders (nested structs/modules), `vis = "pub(crate)"`, and optional `#[derive(...)]` injection into the generated file. |
| **`megamac` playground** | High | Medium | A small binary (or `cargo expand` integration) that prints the expansion of a rsmack-based macro from a snippet — massively lowers the learning curve and helps debugging doc extraction. |
| **`cargo-semver-checks` in CI** | Medium | Low | The workspace publishes 6 crates; adding `cargo-semver-checks` to CI catches accidental breaking changes before release. |
| **MSRV + docs.rs jobs** | Medium | Low | A `cargo hack check --each-feature --rust-version` job plus `[package.metadata.docs.rs]` (all-features) keeps the declared MSRV honest and docs.rs rich. |
| **Fuzz the parsers** | Low | Medium | `cargo-fuzz` targets for `Args::from_list` and the edoc expression parser — proc-macro panics are user-facing compile errors, fuzzing keeps them graceful. |
| **Multi-error `megamac` docs** | Low | Low | When the impls file has several documentation problems, report all of them (collector) instead of failing on the first. |

## Suggested sequencing

1. **Quick wins** (low effort, immediate value): memoized consts, multi-error edoc,
   wrap skip attribute, seanum custom derives, `cargo-semver-checks`.
2. **Structural** (medium effort, pays off long-term): shared exec driver, Logr-only
   error facade, trybuild UI suite, crate split.
3. **Ambitious** (high effort, differentiators): const-expression evaluation,
   playground, migration-codegen.
