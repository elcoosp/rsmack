//! Expansion-time benchmarks of the `seanum` macro implementation.
//!
//! ```text
//! cargo bench -p rsmack-seanum
//! ```

#[path = "../src/impls/seanum.rs"]
// The included impl file's `#[cfg(test)]` unit tests are compiled into the bench
// binary (bench targets build with `cfg(test)`) but stay unreachable from the
// criterion harness, so their helpers read as dead code here only.
#[allow(dead_code)]
mod seanum;

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use darling::FromMeta;
use rsmack_utils::megamac::ExecEnv;

fn args_from(tokens: proc_macro2::TokenStream) -> seanum::Args {
    let meta_list = darling::ast::NestedMeta::parse_meta_list(tokens).expect("meta list parses");
    seanum::Args::from_list(&meta_list).expect("args parse")
}

fn bench_string_enum(c: &mut Criterion) {
    let env = ExecEnv::new("rsmack_seanum::impls");
    c.bench_function("seanum/exec_8_variants_string", |b| {
        b.iter(|| {
            let args = args_from(quote::quote! { rs_type = "String", db_type = "String" });
            let item: syn::ItemEnum = syn::parse_quote! {
                enum UserStatus {
                    Active, Suspended, Closed, Pending, Deleted, Archived, Banned, Unknown,
                }
            };
            seanum::exec(args, black_box(item), &env)
        });
    });
}

fn bench_integer_enum_with_try_from(c: &mut Criterion) {
    let env = ExecEnv::new("rsmack_seanum::impls");
    c.bench_function("seanum/exec_8_variants_integer_migration", |b| {
        b.iter(|| {
            let args = args_from(quote::quote! {
                rs_type = "i32", db_type = "Integer", migration = true
            });
            let item: syn::ItemEnum = syn::parse_quote! {
                enum UserStatus {
                    Active, Suspended, Closed, Pending, Deleted, Archived, Banned, Unknown,
                }
            };
            seanum::exec(args, black_box(item), &env)
        });
    });
}

criterion_group!(benches, bench_string_enum, bench_integer_enum_with_try_from,);
criterion_main!(benches);
