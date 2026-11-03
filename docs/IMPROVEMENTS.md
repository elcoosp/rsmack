# rsmack improvements & roadmap

This document tracks the engineering review outcome of the workspace. The 1.0.0 pass was
a full design brainstorm with breaking changes allowed — the decisions and the perf
ledger are recorded in [`docs/DESIGN_v2.md`](DESIGN_v2.md).

## Already implemented in this pass

These items were part of the 0.19/0.20 overhauls and are listed here for context:

- **No more `proc-macro-error2`** (future-incompatible, abandoned): diagnostics are now
  built on `syn::Error` — fatal errors flow through `?`, non-fatal ones are collected in
  `Logr` and flushed by `ExecEnv::finish`. No unwinding, no panic catching, one less
  dependency.
- **All majors bumped**: syn 2→3, darling 0.21→0.24, sea-orm 1→2, fake 4→5,
  stringcase 0.4→1.0, strum 0.27→0.28, edition 2024, resolver 3.
- **Workspace-level dependency management**: every external crate is declared once in
  `[workspace.dependencies]`; versions, edition, license and rust-version live in
  `[workspace.package]` (DRY).
- **Test architecture**: unit tests drive every `exec` with parsed inputs, integration
  tests compile the generated macros (including a dedicated example crate), negative
  paths are covered — 100% line/region/function coverage enforced in CI.

### Implemented in the 1.0.0 pass

**megamac**

- ✅ **Signature-driven payload protocol**: `exec(input: TokenStream, env)` forwards the
  raw payload (DSL macros); `exec(args: Args, env)` darling-parses it. The args ident
  follows the signature; `args = "..."` overrides it (and fatally conflicts with raw
  mode). `Attr`/`Derive` item types stay signature-inferred; derives document no
  payload.
- ✅ **Impl-file analysis cache**: process-global, keyed by `(path, mtime, len, kind,
  args override)`, storing plain data (token spans are not `Send`) — warm expansions pay
  a metadata stat; cached fatals/issues are replayed identically.
- ✅ **Richer generated docs**: the args struct's doc comment becomes the intro
  paragraph, `Option<T>` / `#[darling(default)]` fields are marked `*(optional)*`,
  `doc = "none"` disables docs entirely, and the `# Parameters` section only exists when
  there are parameters.

**edoc**

- ✅ **Typed const evaluation** (`ConstValue`): arithmetic on machine ints/floats,
  comparison by kind, string rendering once at doc assembly; float/bool casts saturate
  and wrap like rustc.
- ✅ **Path-aware constants**: consts are keyed by full module path; qualified references
  resolve exactly, bare names resolve when unique, ambiguity lists sorted candidates.
- ✅ **Enums + item-level docs**: `#[edoc((...))]` documents enum variants and the
  struct/enum itself.

**wrap**

- ✅ **Every type wraps**: references, raw pointers, bare fns, trait objects — the
  artificial unsupported-type restriction and its `Debug`-scraping error are gone.

**seanum**

- ✅ **Minimal default derives**: no hidden `serde`/`fake` requirement; `derives(...)`
  is the opt-in path.
- ✅ **Explicit discriminants**: `A = 10, B` continues from 11, `num_value` and `TryFrom`
  follow the real values, duplicates/unparsables are collected errors that skip `TryFrom`.
- ✅ **`rename_all` + `enum_name`**: fleet-wide case normalization (7 cases) and explicit
  table-name override (also used by the migration helper).

**folder_iso_struct**

- ✅ **Inline emission**: the macro scans `$CARGO_MANIFEST_DIR/src/<folder>` at expansion
  time and emits the mirror directly — no build script, no `OUT_DIR`, no `include!`;
  visibility defaults to the annotated struct's, `derives(...)` injects derives on every
  generated struct. The build-script builder remains in `rsmack-utils::fs`.

**workspace**

- ✅ **`rsmack` facade crate**: every macro behind one feature-gated dependency.
- ✅ **`rsmack-utils` v2**: `ExecEnv::new`/`Logr::new` (builder machinery dropped from
  the hot path), raw-payload drivers, fallible `scan_modules`/`ModuleTree::render`
  shared between the macro and build scripts, shared `staleness_key` helper.

## Performance (remaining)

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Cache megamac impl-file **source** reads | Low | Low | The analysis cache already skips the parse; only the read remains on the miss path. |
| Precompute `pascal_options` once per process | Low | Low | Constructed per folder scan; a `OnceLock` would shave nanoseconds. |

## Modularity & DRY & KISS (remaining)

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Split `rsmack-utils` by concern | Medium | Medium | `exec` (macro boilerplate) and `fs` (build-time codegen) have disjoint users. The facade already solves the consumer-side friction; the split would break `rsmack_utils::` paths embedded in generated code — keep deferring. |

## Complementary features backlog

| Idea | Impact | Effort | Notes |
| --- | --- | --- | --- |
| Fuzzing in CI (scheduled) | Medium | Medium | Run `cargo fuzz build` + short fuzz runs on a weekly schedule; the targets exist in `fuzz/`, a scheduled job keeps regressions from landing. |
| `edoc` richer const-eval (structs, `include!`) | Low | High | The evaluator covers the documented subset; full rustc const-eval parity would need a real interpreter or `const_eval` crate integration. |
| `seanum` const-expression discriminants | Low | Medium | Discriminants currently accept integer literals (optionally negated); honoring arbitrary const expressions needs resolving user consts, which proc macros cannot evaluate. |
| `megamac` playground expansion of *macro invocations* (not just entrypoints) | Medium | High | Printing the expansion of `greeting!(who = "x")` requires executing the user's `exec` in a sandbox — only realistic via `cargo expand` integration. |
| Per-nested-struct derives for `folder_iso_struct` | Low | Low | `derives(...)` applies uniformly; per-depth derives would need a dedicated option shape. |

## Suggested sequencing

1. **Quick wins**: scheduled fuzzing job, per-process `pascal_options`.
2. **Structural**: `rsmack-utils` crate split (when the generated-path break is
   acceptable).
3. **Ambitious**: full const-eval parity, invocation-level playground.
