//! Expansion-time benchmarks of the `megamac` macro generator.
//!
//! Measures the full generation pipeline: impl file read + parse, `Args` doc
//! extraction and entrypoint emission (using this crate's real `impls/megamac.rs`).
//!
//! ```text
//! cargo bench -p rsmack-megamac
//! ```

#[path = "../src/impls/megamac.rs"]
// The included impl file's `#[cfg(test)]` unit tests are compiled into the bench
// binary (bench targets build with `cfg(test)`) but stay unreachable from the
// criterion harness, so their helpers read as dead code here only.
#[allow(dead_code)]
mod megamac_impls;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use darling::FromMeta;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};

fn args_from(tokens: proc_macro2::TokenStream) -> megamac_impls::Args {
    let meta_list = darling::ast::NestedMeta::parse_meta_list(tokens).expect("meta list parses");
    megamac_impls::Args::from_list(&meta_list).expect("args parse")
}

fn bench_func_generation(c: &mut Criterion) {
    let env = ExecEnv::builder("rsmack_megamac::impls").build();
    let args = args_from(quote::quote! { kind = "Func", name = "megamac" });
    c.bench_function("megamac/exec_func_generation", |b| {
        b.iter(|| megamac_impls::run(black_box(&args), &env, black_box(&package_src_folder())));
    });
}

fn bench_attr_generation(c: &mut Criterion) {
    let env = ExecEnv::builder("rsmack_megamac::impls").build();
    let args = args_from(quote::quote! { kind = "Attr", name = "megamac" });
    c.bench_function("megamac/exec_attr_generation", |b| {
        b.iter(|| megamac_impls::run(black_box(&args), &env, black_box(&package_src_folder())));
    });
}

criterion_group!(benches, bench_func_generation, bench_attr_generation);
criterion_main!(benches);
