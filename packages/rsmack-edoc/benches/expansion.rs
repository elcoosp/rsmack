//! Expansion-time benchmarks of the `edoc` macro implementation.
//!
//! The proc-macro crate type cannot export internals, so the implementation modules
//! are included directly: the measured code is byte-for-byte the shipped one.
//!
//! ```text
//! cargo bench -p rsmack-edoc
//! ```

// The included impl file's `#[cfg(test)]` unit tests are compiled into the bench
// binary (bench targets build with `cfg(test)`) but stay unreachable from the
// criterion harness, so their helpers read as dead code here only.
#[allow(dead_code)]
mod impls;

use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use darling::FromMeta;
use impls::const_eval::Consts;
use impls::edoc::{Args, exec, parse_consts_source};
use rsmack_utils::megamac::ExecEnv;

/// Build the `#[edoc(from = ...)]` args of the bench struct.
fn args_from() -> Args {
    let meta_list = darling::ast::NestedMeta::parse_meta_list(quote::quote! { from = consts })
        .expect("meta list parses");
    Args::from_list(&meta_list).expect("args parse")
}

/// The struct under measurement: 8 documented fields mixing literals and const refs.
fn bench_struct() -> syn::Item {
    syn::parse_quote! {
        struct Client {
            #[edoc(("App: ", APP, " v", VERSION))]
            name: String,
            #[edoc(("Verbose: ", VERBOSE))]
            verbose: bool,
            #[edoc(("Greeting: ", GREETING))]
            greeting: String,
            #[edoc(("Tag: ", TAG))]
            tag: String,
            #[edoc(("Deep: ", nested::DEEP))]
            deep: String,
            #[edoc(("Magic: ", MAGIC))]
            magic: String,
            #[edoc(("Literal ", "fallback"))]
            literal: String,
            #[edoc(("Arith: ", 1 + 1))]
            arith: String,
        }
    }
}

/// Constants file exercising every supported const expression kind.
const CONSTS_SOURCE: &str = r#"
pub const APP: &str = "myapp";
pub const VERSION: &str = "1.2.3";
pub const VERBOSE: bool = true;
pub const MAGIC: &[u8] = b"ab";
pub const GREETING: &str = concat!("hello", " ", "world");
pub const MAJOR: i32 = 1;
pub const TAG: &str = concat!("v", MAJOR);
mod nested {
    pub const DEEP: &str = "deep";
}
"#;

fn bench_consts_parsing(c: &mut Criterion) {
    c.bench_function("edoc/consts_parse_and_eval", |b| {
        b.iter(|| parse_consts_source(black_box(CONSTS_SOURCE)).expect("consts parse"));
    });
}

fn bench_exec_warm_cache(c: &mut Criterion) {
    // Seed the process-global consts cache once, then measure pure expansion.
    let root = std::env::temp_dir().join("rsmack-edoc-bench");
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create bench src dir");
    std::fs::write(src.join("consts.rs"), CONSTS_SOURCE).expect("write consts file");
    // SAFETY: benches are single-threaded binaries
    unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
    let env = ExecEnv::new("rsmack_edoc::impls");
    let warmup = exec(args_from(), bench_struct(), &env);
    assert!(!warmup.to_string().contains("compile_error"));

    c.bench_function("edoc/exec_warm_consts_cache", |b| {
        b.iter(|| exec(args_from(), black_box(bench_struct()), &env));
    });
    std::fs::remove_dir_all(&root).ok();
}

fn bench_const_eval(c: &mut Criterion) {
    let consts = Consts::new();
    let expr: syn::Expr =
        syn::parse2(quote::quote! { concat!("v", 1 + 2, "-", 3 * 4) }).expect("expr parses");
    c.bench_function("edoc/const_eval_expression", |b| {
        b.iter(|| {
            impls::const_eval::eval(black_box(&expr), black_box(&consts), &mut |_| {
                Err("unresolved".to_string())
            })
            .expect("const eval")
        });
    });
}

fn bench_missing_consts_file(c: &mut Criterion) {
    // Error path: metadata failure dominates, worth keeping cheap.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // SAFETY: benches are single-threaded binaries
    unsafe { std::env::set_var("CARGO_MANIFEST_DIR", root) };
    let env = ExecEnv::new("rsmack_edoc::impls");
    c.bench_function("edoc/exec_missing_consts_file", |b| {
        b.iter(|| exec(args_from(), black_box(bench_struct()), &env).to_string());
    });
}

criterion_group!(
    benches,
    bench_consts_parsing,
    bench_exec_warm_cache,
    bench_const_eval,
    bench_missing_consts_file,
);
criterion_main!(benches);
