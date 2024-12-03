use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rsmack_utils::logr::Logr;
use syn::spanned::Spanned;
use syn::*;
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Name of the crate, expected to be in the CARGO_MANIFEST_DIR
    from_crate: Ident,
    /// Folder name, should be flat
    folder: Ident,
}

/// Execute folder_iso_struct macro
pub fn exec(args: Args, item: ItemStruct, logr: Logr) -> TokenStream {
    let from_crate = args.from_crate.to_token_stream().to_string();
    let folder = args.folder.to_token_stream().to_string();
    let mut transformed_item = item.clone();

    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => syn::Fields::Named(fields_named),
        _ => logr.abort(
            transformed_item.fields.span(),
            "Only named struct supported",
        ),
    };
    let name = item.ident.clone();
    let name_str = name.to_string();
    let folder_str = folder.to_string();
    let attrs = item.attrs.clone();
    let generics = item.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {

        #item

        impl #impl_generics #name #ty_generics #where_clause {
            /// Build time generate this struct by calling [rsmack_utils::folder_iso_struct]")]
            fn generate() -> () {
                rsmack_utils::folder_iso_struct()
                    .pre(quote::quote! { #(#attrs)* })
                    .name(#name_str)
                    .from_crate(#from_crate)
                    .folder(#folder_str)
                    .call();
            }
        }

    }
}
