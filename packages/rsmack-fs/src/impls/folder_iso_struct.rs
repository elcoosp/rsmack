/// A procedural macro for generating structs that mirror folder structures at compile time.
/// This macro creates a struct whose fields correspond to files in a specified folder,
/// enabling type-safe access to file contents and metadata.
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;
use syn::*;

/// Arguments for the `folder_iso_struct` macro
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Name of the crate containing the source folder.
    /// Expected to be located within `CARGO_MANIFEST_DIR`.
    from_crate: syn::Ident,

    /// Name of the folder to mirror in the struct.
    /// Should be a flat directory (no subdirectories).
    folder: syn::Ident,
}

/// Executes the `folder_iso_struct` macro transformation
///
/// # Arguments
/// * `args` - Parsed macro arguments specifying source crate and folder
/// * `item` - The input struct to transform
/// * `env` - Macro execution environment for error reporting
///
/// # Returns
/// Returns a `TokenStream` containing:
/// 1. The original struct definition
/// 2. An implementation with a `generate()` method that creates the folder structure
///    at compile time using `rsmack_utils::fs::folder_iso_struct`
///
/// # Behavior
/// - Validates that the input struct uses named fields
/// - Generates a `generate()` method that will be executed at build time
/// - The generated method uses the provided crate and folder names to create
///   a struct mirroring the folder structure
pub fn exec(args: Args, item: ItemStruct, env: ExecEnv) -> TokenStream {
    let from_crate = args.from_crate.to_token_stream().to_string();
    let folder = args.folder.to_token_stream().to_string();
    let mut transformed_item = item.clone();

    // Ensure we're only working with structs that have named fields
    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => syn::Fields::Named(fields_named),
        _ => env.logr.abort(
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
            /// Generates the folder structure at compile time.
            ///
            /// This method is automatically called during the build process to create
            /// a struct representation of the specified folder. It uses the provided
            /// crate name and folder path to locate and process the directory structure.
            fn generate() -> () {
                rsmack_utils::fs::folder_iso_struct()
                    .pre(&quote::quote! { #(#attrs)* })
                    .name(#name_str)
                    .from_crate(#from_crate)
                    .folder(#folder_str)
                    .call();
            }
        }
    }
}
