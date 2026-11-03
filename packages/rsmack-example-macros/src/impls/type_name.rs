//! Derive macro: implements a `TypeName` trait returning the type name.
//!
//! v2: derive macros receive no payload, so there is no `Args` struct and no
//! `# Parameters` section — the derive needs only the `exec` function.

use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

pub fn exec(item: syn::DeriveInput, env: &ExecEnv) -> TokenStream {
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
