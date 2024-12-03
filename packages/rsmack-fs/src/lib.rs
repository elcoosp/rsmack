use proc_macro::TokenStream;
use proc_macro_error2::*;
use rsmack_utils::*;
mod impls;
#[proc_macro_error]
#[proc_macro_attribute]
/// Create a struct from a flat folder which filenames are struct fields, and types `$filename::<title_case($filename)>`
pub fn folder_iso_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    exec::call_impls_with_args!(folder_iso_struct, syn::ItemStruct, attr, item)
}
