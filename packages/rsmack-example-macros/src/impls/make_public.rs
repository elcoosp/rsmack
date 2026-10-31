//! Attribute macro: makes every field of the struct public.

use darling::FromMeta;
use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

/// Make-public configuration (currently empty)
#[derive(Debug, FromMeta)]
pub struct Args;

pub fn exec(_args: Args, mut item: syn::ItemStruct, env: ExecEnv) -> TokenStream {
    for field in &mut item.fields {
        field.vis = syn::Visibility::Public(syn::token::Pub::default());
    }
    env.finish(quote::quote! { #item })
}
