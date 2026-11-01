# Fuzzing rsmack parsers

Proc-macro panics are user-facing compile errors, so the parsers behind rsmack are
fuzzed to keep them graceful:

- `edoc_consts_source` — the constants-file parser + mini const evaluator
  (`parse_consts_source`).
- `edoc_args_meta` / `wrap_args_meta` / `megamac_args_meta` — the darling
  `Args::from_list` meta parsers of every macro.

## Run

```sh
cargo install cargo-fuzz
cargo fuzz run -j 4 --parallelism 1 edoc_consts_source -- -max_len=4096
cargo fuzz run edoc_args_meta
```

`cargo fuzz build` requires the nightly toolchain. The targets include the proc-macro
implementation sources through `#[path]` (proc-macro crates cannot export plain
items), so they always measure the shipped code.
