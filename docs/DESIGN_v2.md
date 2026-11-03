# rsmack v2 — design (the "no backward compat" pass)

This document is the brainstorm and design record of the v2 overhaul. Every change here
is a deliberate, breaking decision: the goal is a finished product that beats the
alternatives on **usages**, **DX** and **performance**, not a compatible increment.

## North star

> Any Rust developer authors a production-grade, fully documented, panic-free proc
> macro in under twenty lines, and installs every shipped macro with one
> `cargo add rsmack`.

Three axes, one release: **v1.0.0** across the workspace.

---

## 1. megamac v2 — the universal macro factory

### Problem today
- A `megamac`-generated **function-like** macro can only accept darling meta syntax
  (`key = value`). DSL-style macros (`sql!(SELECT ...)`, `html!(<div> ...)`) are
  impossible — the single biggest usage gap versus writing a macro by hand.
- The impl file is re-read and re-parsed on **every** expansion (~1.35 ms, and
  rust-analyzer re-expands constantly).
- The parameter docs are thin: no optional markers, the `Args` struct's own doc comment
  is silently dropped.

### Design
1. **Signature-driven payload protocol** — the `exec` signature *is* the configuration:
   - `exec(args: Args, env: &ExecEnv)` → darling meta parsing (classic mode).
   - `exec(input: proc_macro2::TokenStream, env: &ExecEnv)` → the raw payload is passed
     through untouched (NEW, `Func` only). Unlocks DSL macros with zero framework cost.
   - The generated entrypoint is chosen by inspecting the impl file's `exec` signature —
     the same inference style `Attr`/`Derive` already use for the item type.
2. **Impl-file cache** — process-global `(path, mtime, len)` keyed memo of the parsed
   `syn::File` **and** the extracted `FieldDoc`s. Cold expansion pays the parse once;
   warm expansions (rust-analyzer re-checks, test loops) pay a metadata stat.
3. **Richer generated docs**:
   - the `Args` struct's own doc comment becomes the intro paragraph;
   - `Option<T>` fields and `#[darling(default)]` fields are marked *optional*;
   - `doc = "none"` joins `readme` | `inline` for doc-less codegen.
4. **No-parameters fast path** — an empty/unit `Args` skips doc extraction entirely.

## 2. edoc v2 — typed const evaluation

### Problem today
- Values are `HashMap<String, String>`: every arithmetic op re-parses strings into
  `i128`/`f64`, and `"a" == "a"` compares *rendered* strings. Slow and subtly wrong for
  bytes/chars.
- Const lookup uses the **last path segment only**: two consts named alike in different
  nested modules collide silently; qualified references are not actually qualified.
- Enums cannot take `#[edoc]`; only struct fields are documented.

### Design
1. **`ConstValue` enum** (`Str`, `Int(i128)`, `Float(f64)`, `Bool`, `Bytes`, `Char`) —
   arithmetic operates on the enum, formatting happens once at doc assembly.
2. **Path-aware consts** — consts are keyed by their full module path
   (`nested::DEEP`). Qualified references resolve exactly; a bare name resolves only
   when unambiguous, otherwise a collected error names the candidates.
3. **Enums + item-level docs** — `#[edoc((...))]` on enum variants produces variant
   docs; on the struct/enum itself it produces the item doc.
4. The memoized constants cache is kept, now storing typed values.

## 3. wrap v2 — wrap everything

### Problem today
Only `Path`/`Slice`/`Tuple`/`Array` field types can be wrapped; `&'static str`,
`*const u8`, `fn(u8) -> bool`, `dyn Trait` are all refused — yet every one of them
wraps perfectly legally (`Option<&str>`, `Box<dyn Trait>`, ...). The kind name in the
error is scraped from `format!("{ty:?}")`.

### Design
- **The restriction is lifted**: any field type wraps. The unsupported-type error path
  is deleted. `matches!`-based handling replaces Debug-string scraping.
- `skip` / `with` / `map` semantics unchanged (they are right).

## 4. seanum v2 — sane defaults, finally

### Problem today
- The generated derive list hardcodes `::fake::Dummy`, `::serde::Serialize`,
  `::serde::Deserialize`: every consumer is forced to depend on `fake` and `serde` even
  for a pure-DB enum. Hidden dependency tax.
- Integer `rs_type` rejects explicit discriminants (roadmap item).
- No `rename_all`, no `enum_name` override.

### Design
1. **Minimal default derives**: `Clone, Debug, PartialEq, Eq, Hash, EnumIter,
   DeriveActiveEnum`. Anything else — `Serialize`, `Deserialize`, `Dummy`, `Display` —
   is opt-in via `derives(...)`. The macro no longer forces dependencies on users.
2. **Explicit discriminants** (integer `rs_type`): `A = 2` works; `num_value` follows
   the discriminant, `TryFrom` matches real values, duplicates are collected errors.
   Non-unit variants without discriminants keep the index fallback with an error only
   when a `TryFrom` would be wrong.
3. **`rename_all = "..."`** (`snake_case`, `camelCase`, `kebab-case`, `PascalCase`,
   `SCREAMING_SNAKE_CASE`, `lowercase`, `UPPERCASE`) applies to every non-renamed
   string value.
4. **`enum_name = "..."`** overrides the snake_cased table name.

## 5. folder_iso v2 — one attribute, zero build script

### Problem today
The flow requires: annotate a struct (whose fields are ignored), call
`Levels::generate()` from a build script, wait for `OUT_DIR`, then `include!` the
generated file — a step the README never even mentions. Three moving parts where one
attribute should suffice.

### Design
1. **Inline emission (default and only mode)**: the macro scans the folder at expansion
   time and emits the mirrored struct **inline** — nested modules for subdirectories
   with `recursive = true`, annotated struct's attributes preserved on every generated
   struct, `derives("...")` injects extra derives.
2. `from_crate` is gone (breaking): the folder resolves from the calling crate's own
   `src/` — the only case that matters for an attribute macro.
3. `rsmack_utils::fs::{folder_iso_struct, generate_file, calling_crate_dir}` remain as
   explicit build-script utilities for authors who want OUT_DIR codegen.

## 6. `rsmack` — the facade crate (new)

Seven crates is seven `cargo add` lines. A feature-gated umbrella crate
`packages/rsmack` re-exports every macro:

```toml
[dependencies]
rsmack = "1"                      # everything
rsmack = { version = "1", default-features = false, features = ["edoc"] }
```

Individual crates stay published for precise dependency control.

## 7. Core (rsmack-utils)

- `ExecEnv::new(prefix)` — the bon builder is dropped for the env (a two-field struct
  does not need a builder); bon stays for the `fs` codegen builder.
- Shared `fs::staleness_key(path)` helper powering both caches (edoc + megamac).
- `Logr` and the entrypoint drivers are proven; they gain the raw-payload driver.

---

## Performance ledger (expected)

| Change | Win |
| --- | --- |
| Typed const eval (edoc) | no per-op string parse/format; arithmetic on machine ints |
| Impl-file cache (megamac) | ~1.3 ms saved on every warm expansion (RA, test loops) |
| No-params fast path (megamac) | doc extraction skipped entirely |
| `matches!` type dispatch (wrap/seanum) | no `format!("{ty:?}")` scraping |
| `ExecEnv::new` | no builder machinery on the hot path of every expansion |

## Compatibility

Deliberately broken (documented in CHANGELOG 1.0.0):

- `seanum`: default derives reduced (`fake`/`serde` now opt-in); `TryFrom` honors
  explicit discriminants.
- `folder_iso_struct`: inline emission; `from_crate` and the `generate()` build-script
  flow removed from the macro (utilities remain).
- `megamac`: `doc = "none"` added; generated entrypoints may pass raw tokens.
- `wrap`: unsupported-type error path removed (everything wraps).
- `rsmack-utils`: `ExecEnv::builder` replaced by `ExecEnv::new`.

## Non-goals (kept from the old roadmap, still out)

- Physical crate split of `rsmack-utils` (the facade solves the consumer-side issue).
- Full rustc const-eval parity (`include!`, struct/enum consts).
- Invocation-level playground (`greeting!(who = "x")` expansion printing).
