# Benchmarks

Expansion-time benchmarks of every rsmack macro, measured with
[criterion](https://docs.rs/criterion) on the shipped implementation sources (the
proc-macro crate type cannot export internals, so `benches/` includes the impl modules
via `#[path]` — the measured code is byte-for-byte the published one).

- Machine: GitHub Actions `ubuntu-latest` class runner, `cargo bench` (release profile).
- Reproduce locally: `cargo bench --workspace` (see `.github/workflows/bench.yml`).
- Reports: `target/criterion/report/index.html` (criterion's HTML report).

## Results

Reference run (2026-11, criterion 0.8.2, mean of 100 samples):

| Benchmark | Mean | Notes |
| --- | --- | --- |
| `utils/exec_env_build` | 35 ns | `ExecEnv` construction (bon builder) |
| `utils/finish_result_ok` | 158 ns | shared exec driver, happy path overhead |
| `utils/logr_collect_4_errors` | 3.9 µs | collect 4 diagnostics + flush to tokens |
| `edoc/const_eval_expression` | 4.5 µs | mini const-eval of `concat!("v", 1 + 2, "-", 3 * 4)` |
| `edoc/consts_parse_and_eval` | 27 µs | constants file parse + evaluation (cold path) |
| `edoc/exec_missing_consts_file` | 27 µs | error path: missing constants file |
| `edoc/exec_warm_consts_cache` | 62 µs | full 8-field expansion with memoized consts |
| `wrap/exec_4_fields` | 16 µs | 4-field struct wrap |
| `wrap/exec_24_fields` | 72 µs | 24-field struct wrap (~3 µs/field, linear) |
| `seanum/exec_8_variants_string` | 19 µs | string `rs_type`, 8 variants |
| `seanum/exec_8_variants_integer_migration` | 51 µs | integer `rs_type` + `TryFrom` + migration helper |
| `megamac/exec_func_generation` | 1.35 ms | impl file read + parse + doc extraction + emit |
| `megamac/exec_attr_generation` | 1.39 ms | same, attribute entrypoint |

## Reading the numbers

- **Memoized constants pay off.** `edoc/consts_parse_and_eval` (~27 µs) is what every
  annotated struct used to pay; with the process-global memo it is paid **once per
  constants file per compilation**, and `edoc/exec_warm_consts_cache` (58 µs for a full
  8-field expansion) shows the warm path. A crate with N structs against the same
  `consts.rs` saves `N × 27 µs` of expansion time.
- **megamac generation is file-I/O bound** (~1.3 ms, dominated by reading and parsing
  the impl file). It runs once per macro per compilation, not per item, so this is
  negligible in practice.
- **Error paths are cheap.** A missing constants file costs the same as a cold parse —
  no pathological behavior when diagnostics fire.
- **Diagnostics are effectively free.** Collecting and flushing 4 errors costs 3.9 µs;
  the shared `finish_result` driver adds ~159 ns to every expansion.
