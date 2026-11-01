//! Benchmarks of the diagnostics collector (`Logr`) and the shared exec driver.
//!
//! ```text
//! cargo bench -p rsmack-utils
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rsmack_utils::logr::Logr;
use rsmack_utils::megamac::ExecEnv;

fn bench_logr_collected_errors(c: &mut Criterion) {
    c.bench_function("utils/logr_collect_4_errors", |b| {
        b.iter(|| {
            let logr = Logr::builder().prefix("bench::macro").build();
            for index in 0..4 {
                logr.emit_error(
                    proc_macro2::Span::call_site(),
                    format!("Collected diagnostic number {index}"),
                );
            }
            logr.finish(black_box(quote::quote! { struct Demo; }));
        });
    });
}

fn bench_finish_result_ok(c: &mut Criterion) {
    let env = ExecEnv::builder("bench::macro").build();
    c.bench_function("utils/finish_result_ok", |b| {
        b.iter(|| env.finish_result(Ok(black_box(quote::quote! { struct Demo; }))));
    });
}

fn bench_exec_env_build(c: &mut Criterion) {
    c.bench_function("utils/exec_env_build", |b| {
        b.iter(|| ExecEnv::builder(black_box("bench::macro::demo")).build());
    });
}

criterion_group!(
    benches,
    bench_logr_collected_errors,
    bench_finish_result_ok,
    bench_exec_env_build,
);
criterion_main!(benches);
