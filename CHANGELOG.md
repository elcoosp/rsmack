# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [1.0.0] - 2026-11-03

### Features

- [**breaking**] Signature-driven payload protocol for `megamac`: an `exec` taking a
  `proc_macro2::TokenStream` first parameter receives the raw payload (DSL macros),
  otherwise the payload is darling-parsed into the documented args struct; the args
  ident follows the `exec` signature and `doc = "none"` joins `readme`/`inline`

- [**breaking**] New `rsmack` facade crate: every macro behind one feature-gated
  dependency (`cargo add rsmack`)

- Megamac impl-file analysis cache (process-global, `(path, mtime, len)` keyed, plain
  data only), richer generated docs (args-struct intro paragraph, `*(optional)*` markers
  from `Option<T>`/`#[darling(default)]`, no `# Parameters` section when there are no
  parameters) and derive macros no longer document a payload

- [**breaking**] Typed const-eval for `edoc` (`ConstValue`: str/int/float/bool/bytes/
  char), path-aware constants (full module paths, unique bare-name fallback, ambiguity
  errors with sorted candidates), enum variant docs and item-level docs, float/bool cast
  support, deterministic diagnostics

- [**breaking**] `wrap` wraps **every** field type (references, raw pointers, bare fns,
  trait objects); the unsupported-type error path is gone

- [**breaking**] `seanum` minimal default derives (no hidden `serde`/`fake`),
  explicit integer discriminants with duplicate detection, `rename_all = "..."` and
  `enum_name = "..."` options

- [**breaking**] `folder_iso_struct` emits the mirrored struct **inline** at expansion
  time (no build script, no `OUT_DIR`, no `include!`); `from_crate` and the
  `generate()` flow are gone, `derives(...)` is new, visibility defaults to the
  annotated struct's

- `rsmack-utils`: `ExecEnv::new`/`Logr::new` constructors replace the bon builders,
  raw-payload entrypoint drivers (`func_raw_entrypoint`, `attr_raw_entrypoint`),
  fallible `scan_modules`/`ModuleTree::render` shared by the macro and build scripts,
  shared `staleness_key` cache helper

### Documentation

- Rewrite the workspace README and every crate README for the v2 protocol

- Add docs/DESIGN_v2.md (brainstorm, breaking decisions, perf ledger)

### Performance

- Megamac warm expansions pay a metadata stat instead of a full read + parse
  (~1.3 ms saved per re-expansion)

- Edoc const arithmetic operates on machine integers/floats; doc assembly formats once

## [0.20.0] - 2026-11-01

### Features

- [**breaking**] Migrate to syn 3, darling 0.24, edition 2024 and drop proc-macro-error2

- Add rsmack-example-macros and reach 100% test coverage

- [**breaking**] Single-source the proc-macro entrypoint drivers

- Edoc const-expression evaluation, memoized consts and multi-error reporting

- Wrap field attributes, seanum customizations and recursive folder mirroring

- Megamac playground, custom impls/args idents and inline docs

- Fuzz the parsers with cargo-fuzz


### Documentation

- Remake README with logo, refresh crate docs and add improvements roadmap

- Refresh README, add error style guide and update the roadmap


### Testing

- Appease pedantic lints in test files

- Snapshot every error path with trybuild


### Miscellaneous Tasks

- Add CI matrix, git-cliff changelog and tag driven release

- Semver checks, MSRV job and UI tests; fix branch filter

- Bump workspace version to 0.20.0


### Bench

- Criterion expansion benchmarks for every macro

## [0.18.0] - 2025-11-16

### Bug Fixes

- *(deps)* Update rust crate build-print to v1

- Readd deps for doc tests

## [0.17.0] - 2025-11-16

### Bug Fixes

- Rm sea-orm serde & fake dep, ref from root inside derive


### Miscellaneous Tasks

- 0.17.0

## [0.16.0] - 2025-11-14

### Features

- Collect existing attributes


### Miscellaneous Tasks

- 0.16.0

## [0.15.0] - 2025-11-13

### Bug Fixes

- Derive Hash for seanum


### Documentation

- Fix doc versions


### Miscellaneous Tasks

- V0.15.0

## [0.14.0] - 2025-11-12

### Bug Fixes

- Fix invalid documentation


### Miscellaneous Tasks

- 0.14.0

## [0.13.0] - 2025-11-12

### Features

- *(seanum)* Implement seanum, generate a Clone, Dummy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Eq, Serialize, Deserialize enum wired with correct attrs mirroring the enum

- #[doc = include_str!(../README.md)] for each macro


### Bug Fixes

- Version in cargo.toml

- *(deps)* Update rust crate darling to 0.21.0

- *(deps)* Update rust crate stringcase to 0.4

- *(deps)* Update rust crate strum to 0.27.0


### Documentation

- Fix readme install

- Improve megamac readme

- Megamac readme fix incorrect doc

- Add seanum doc


### Miscellaneous Tasks

- Up softprops/action-gh-release@v2

- 0.13.0

## [0.11.0] - 2025-11-07

### Bug Fixes

- *(wrap)* License field

## [0.10.0] - 2025-11-07

### Bug Fixes

- Missing version on dep

## [0.9.0] - 2025-11-07

### Bug Fixes

- Missing description & license

## [0.8.0] - 2025-11-07

### Miscellaneous Tasks

- Use katyo/publish-crates@v2

## [0.7.0] - 2025-11-07

### Bug Fixes

- Version in cargo.toml

## [0.6.0] - 2025-11-07

### Bug Fixes

- Run clippy display fix

## [0.5.0] - 2025-11-07

### Bug Fixes

- Run clippy


### Miscellaneous Tasks

- Test then release

## [0.3.0] - 2025-11-07

### Bug Fixes

- Up cargo.toml v


### Miscellaneous Tasks

- Rm test from release workflow

- V0.3.0

## [0.2.0] - 2025-11-07

### Bug Fixes

- Up cargo.toml version


### Miscellaneous Tasks

- Fix on.push.tags

## [0.1.0] - 2025-11-07

### Features

- Wrap macro to wrap all fields ty of a named struct in the given ty wrapper ident

- Create rsmack-utils to reuse call_impls_with_args macro_export

- *(rsmack-utils)* Folder_iso_struct builder proc_macro do nothing for now

- Better error handling for impls using exec via Logr struct

- Add Logr::new_module_path

- Complement logr, use own file for exec & use proc-macro-error2

- Add megamac crate to remove remove proc-macro decl boilerplate

- Add way to get the call site source file via unstable, pass env struct to exec

- Split get_args_fields_doc

- Split get_macro_impl_file_ast & auto gen macro doc with megamac

- *(megamac)* Stable base derive support

- Working edoc in its own crate non derive but use DeriveInput to collect field attrs under the hood

- *(utils)* Use cl macros in logr

- *(utils)* Add schemars::impl_json_schema


### Bug Fixes

- Rename vars

- *(utils)* Split fs mod

- *(fs)* Incorrect path fs mod

- *(megamac)* Revert use of procmacro2_semver_exempt to get sourcefile instead derive package_src_folder from std::env::var (not env! because it is compile time)

- Some clippy lints

- *(utils)* Make logr take M: AsRef<str>

- Run clippy cf logr change

- Unused vars

- Fix all clippy warns

- *(utils)* Split build_env macro


### Documentation

- *(utils)* Fs mod

- Enhance

- Add impls doc

- Add README


### Miscellaneous Tasks

- Comment unused

- Run clippy pedantic

- *(utils)* Rename

- Add publish workflow

<!-- generated by git-cliff -->
