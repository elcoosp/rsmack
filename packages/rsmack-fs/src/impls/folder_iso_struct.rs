/// A procedural macro for generating structs that mirror folder structures at compile time.
/// This macro creates a struct whose fields correspond to files in a specified folder,
/// enabling type-safe access to file contents and metadata.
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;
use syn::*;

/// Arguments for the `folder_iso_struct` macro
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Relative path of the crate containing the source folder, resolved from the
    /// workspace root directory (e.g. `"my_crate"` or `"target/my-test-crate"`).
    from_crate: String,

    /// Name of the folder to mirror in the struct.
    /// Should be a flat directory (no subdirectories).
    folder: String,
}

/// Executes the `folder_iso_struct` macro transformation
///
/// # Returns
/// Returns a `TokenStream` containing:
/// 1. The original struct definition
/// 2. An implementation with a `generate()` method that creates the folder structure
///    at compile time using `rsmack_utils::fs::folder_iso_struct`
pub fn exec(args: Args, item: ItemStruct, env: ExecEnv) -> TokenStream {
    match run(args, item, &env) {
        Ok(output) => env.finish(output),
        Err(error) => env.finish(error.to_compile_error()),
    }
}

fn run(args: Args, item: ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let from_crate = args.from_crate;
    let folder = args.folder;

    // Ensure we're only working with structs that have named fields
    if !matches!(item.fields, syn::Fields::Named(_)) {
        return Err(env
            .logr
            .error(item.fields.span(), "Only named struct supported"));
    }

    let name = item.ident.clone();
    let name_str = name.to_string();
    let folder_str = folder.clone();
    let attrs = item.attrs.clone();
    let generics = item.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #item

        impl #impl_generics #name #ty_generics #where_clause {
            /// Generates the folder structure at compile time.
            ///
            /// This method is automatically called during the build process to create
            /// a struct representation of the specified folder. It uses the provided
            /// crate name and folder path to locate and process the directory structure.
            fn generate() {
                rsmack_utils::fs::folder_iso_struct()
                    .pre(&quote::quote! { #(#attrs)* })
                    .name(#name_str)
                    .from_crate(#from_crate)
                    .folder(#folder_str)
                    .call();
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;
    use quote::quote;

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    fn env() -> ExecEnv {
        ExecEnv::builder("rsmack_fs::impls", "impls", "Args", "folder_iso_struct").build()
    }

    #[test]
    fn generates_struct_with_generate_method() {
        let args = args_from(quote! { from_crate = "my_crate", folder = "levels" });
        let item: ItemStruct = syn::parse_quote! {
            #[derive(Debug)]
            struct Levels {
                alpha: Alpha,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("struct Levels"));
        assert!(output.contains("fn generate ()"));
        assert!(output.contains("rsmack_utils :: fs :: folder_iso_struct ()"));
        assert!(output.contains(". name (\"Levels\")"));
        assert!(output.contains(". from_crate (\"my_crate\")"));
        assert!(output.contains(". folder (\"levels\")"));
        assert!(output.contains(". pre (& quote :: quote ! { # [derive (Debug)] })"));
    }

    #[test]
    fn tuple_struct_is_a_fatal_error() {
        let args = args_from(quote! { from_crate = "my_crate", folder = "levels" });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels(Alpha);
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Only named struct supported"));
    }
}
