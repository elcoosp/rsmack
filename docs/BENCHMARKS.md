# Benchmarks

Expansion-time benchmarks of every rsmack macro, measured with
[criterion](https://docs.rs/criterion) on the shipped implementation sources (the
proc-macro crate type cannot export internals, so `benches/` includes the impl modules
via `#[path]` — the measured code is byte-for-byte the published one).

- Machine: reference numbers from GitHub Actions `ubuntu-latest`-class runners; the
  1.0.0 table below was measured on the release machine (`cargo bench`, criterion 0.8.2,
  mean of 100 samples, Rust 1.99).
- Reproduce locally: `cargo bench --workspace`.
- Reports: `target/criterion/report/index.html` (criterion's HTML report).

## Results (1.0.0)

| Benchmark | Mean | Notes |
| --- | --- | --- |
| `utils/exec_env_build` | 37 ns | `ExecEnv::new` construction |
| `utils/finish_result_ok` | 161 ns | shared exec driver, happy path overhead |
| `utils/logr_collect_4_errors` | 4.1 µs | collect 4 diagnostics + flush to tokens |
| `edoc/const_eval_expression` | 4.6 µs | typed const-eval of `concat!("v", 1 + 2, "-", 3 * 4)` |
| `edoc/consts_parse_and_eval` | 28 µs | constants file parse + evaluation (cold path) |
| `edoc/exec_missing_consts_file` | 27 µs | error path: missing constants file |
| `edoc/exec_warm_consts_cache` | 63 µs | full 8-field expansion with memoized consts |
| `wrap/exec_4_fields` | 18 µs | 4-field struct wrap |
| `wrap/exec_24_fields` | 73 µs | 24-field struct wrap (~3 µs/field, linear) |
| `seanum/exec_8_variants_string` | 23 µs | string `rs_type`, 8 variants, discriminant bookkeeping |
| `seanum/exec_8_variants_integer_migration` | 55 µs | integer `rs_type` + `TryFrom` + migration helper |
| `megamac/exec_func_generation` | **15 µs** | warm: impl-file analysis served from the cache |
| `megamac/exec_attr_generation` | **18 µs** | warm: same, attribute entrypoint |

## Reading the numbers

- **The megamac analysis cache is the headline.** The 0.20 reference run measured
  `megamac/exec_*_generation` at ~1.35 ms (impl file read + parse + doc extraction per
  expansion). With the process-global `(path, mtime, len, kind, args)` memo the *warm*
  expansion is **15–18 µs — roughly a 90× improvement** for every re-expansion of the
  same declaration: rust-analyzer re-checks, test loops, workspace rebuilds. The cold
  path (first expansion per file per process) still pays the one-time ~1.3 ms read +
  parse, and it is paid once per macro declaration per compilation.
- **Memoized constants pay off.** `edoc/consts_parse_and_eval` (~28 µs) is what every
  annotated item used to pay; with the process-global memo it is paid **once per
  constants file per compilation**, and `edoc/exec_warm_consts_cache` (63 µs for a full
  8-field expansion) shows the warm path. A crate with N structs against the same
  `consts.rs` saves `N × 28 µs` of expansion time.
- **Typed const-eval keeps the evaluator flat.** The mini interpreter now carries
  `i128`/`f64`/`bool`/bytes/char values instead of rendering strings between every
  operation; `edoc/const_eval_expression` stays at ~4.6 µs while gaining float casts,
  bool arithmetic errors and path-aware resolution.
- **Error paths are cheap.** A missing constants file costs the same as a cold parse —
  no pathological behavior when diagnostics fire.
- **Diagnostics are effectively free.** Collecting and flushing 4 errors costs ~4 µs;
  the shared `finish_result` driver adds ~160 ns to every expansion.
