use darling::FromMeta;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::{quote, ToTokens};
use rsmack_utils::calling_crate_dir;
use syn::spanned::Spanned;
use syn::*;
#[derive(Debug, FromMeta)]
pub struct Args {
    from_crate: Ident,
    /// Folder name, should be flat
    with: Ident,
}

/// Execute folder_iso_struct macro
pub fn exec(args: Args, item: ItemStruct) -> TokenStream {
    let mut transformed_item = item.clone();
    let ccd = calling_crate_dir();
    let from_crate = args.from_crate.to_token_stream().to_string();
    let folder = args.with.to_token_stream().to_string();
    // Can not call fs
    let _mods_folder_path = ccd.join(from_crate).join(&folder).join("src");
    // abort!(item.span(), format!("{mods_folder_path:#?}"));
    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => {
            let transformed_fields_named = fields_named.clone();

            syn::Fields::Named(transformed_fields_named)
        }
        _ => abort!(
            transformed_item.fields.span(),
            "Only named struct supported"
        ),
    };
    // let name = transformed_item.ident.clone();
    // let generics = transformed_item.generics.clone();
    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {

        #transformed_item

    }
}
