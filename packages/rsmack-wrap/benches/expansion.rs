//! Expansion-time benchmarks of the `wrap` macro implementation.
//!
//! ```text
//! cargo bench -p rsmack-wrap
//! ```

#[path = "../src/impls/wrap.rs"]
// The included impl file's `#[cfg(test)]` unit tests are compiled into the bench
// binary (bench targets build with `cfg(test)`) but stay unreachable from the
// criterion harness, so their helpers read as dead code here only.
#[allow(dead_code)]
mod wrap;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use darling::FromMeta;
use rsmack_utils::megamac::ExecEnv;

fn args_from(tokens: proc_macro2::TokenStream) -> wrap::Args {
    let meta_list = darling::ast::NestedMeta::parse_meta_list(tokens).expect("meta list parses");
    wrap::Args::from_list(&meta_list).expect("args parse")
}

fn bench_small_struct(c: &mut Criterion) {
    let env = ExecEnv::builder("rsmack_wrap::impls").build();
    c.bench_function("wrap/exec_4_fields", |b| {
        b.iter(|| {
            let item: syn::ItemStruct = syn::parse_quote! {
                struct User {
                    name: String,
                    tags: Vec<String>,
                    point: (u8, u8),
                    nums: [u8; 2],
                }
            };
            exec_wrapper(black_box(item), &env);
        });
    });
}

fn bench_large_struct(c: &mut Criterion) {
    let env = ExecEnv::builder("rsmack_wrap::impls").build();
    c.bench_function("wrap/exec_24_fields", |b| {
        b.iter(|| {
            let item: syn::ItemStruct = syn::parse_quote! {
                struct Big {
                    f0: String, f1: u8, f2: u16, f3: u32, f4: u64, f5: Vec<String>,
                    f6: String, f7: u8, f8: u16, f9: u32, f10: u64, f11: Vec<String>,
                    f12: String, f13: u8, f14: u16, f15: u32, f16: u64, f17: Vec<String>,
                    f18: String, f19: u8, f20: u16, f21: u32, f22: u64, f23: (u8, u8),
                }
            };
            exec_wrapper(black_box(item), &env);
        });
    });
}

fn exec_wrapper(item: syn::ItemStruct, env: &ExecEnv) -> proc_macro2::TokenStream {
    wrap::exec(args_from(quote::quote! { with = "Option" }), item, env)
}

criterion_group!(benches, bench_small_struct, bench_large_struct);
criterion_main!(benches);
