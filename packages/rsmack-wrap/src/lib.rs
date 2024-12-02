use proc_macro::TokenStream;
use proc_macro_error::*;
use rsmack_utils::*;
mod impls;
#[proc_macro_error]
#[proc_macro_attribute]
/// Wrap given (named) struct fields into given `args.with`
pub fn wrap(attr: TokenStream, item: TokenStream) -> TokenStream {
    exec::call_impls_with_args!(wrap, syn::ItemStruct, attr, item)
}
