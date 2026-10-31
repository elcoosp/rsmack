//! Derive macro: implements a `TypeName` trait returning the type name.

use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

/// Type-name configuration (currently empty)
// Never constructed: derive macros receive no attribute meta, but `megamac` still reads
// this documented `Args` struct to generate the `# Parameters` doc section.
#[allow(dead_code)]
pub struct Args;

pub fn exec(item: syn::DeriveInput, env: ExecEnv) -> TokenStream {
    let ident = item.ident;
    let name = ident.to_string();
    env.finish(quote::quote! {
        impl TypeName for #ident {
            fn type_name() -> &'static str {
                #name
            }
        }
    })
}
