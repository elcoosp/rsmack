/// A procedural macro for generating structs that mirror folder structures at compile time.
/// This macro creates a struct whose fields correspond to files in a specified folder,
/// enabling type-safe access to file contents and metadata.
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;
use syn::{ItemStruct, Visibility};

/// Arguments for the `folder_iso_struct` macro
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Relative path of the crate containing the source folder, resolved from the
    /// workspace root directory (e.g. `"my_crate"` or `"target/my-test-crate"`).
    from_crate: String,

    /// Name of the folder to mirror in the struct.
    /// Should be a flat directory (no subdirectories) unless `recursive = true`.
    folder: String,

    /// Visibility of the generated struct and modules (e.g. `"pub"`, `"pub(crate)"`).
    #[darling(default)]
    vis: Option<String>,

    /// Mirror subdirectories as nested modules with their own struct
    /// (`recursive = true`).
    #[darling(default)]
    recursive: bool,
}

/// Executes the `folder_iso_struct` macro transformation
///
/// # Returns
/// Returns a `TokenStream` containing:
/// 1. The original struct definition
/// 2. An implementation with a `generate()` method that creates the folder structure
///    at compile time using `rsmack_utils::fs::folder_iso_struct`
pub fn exec(args: Args, item: ItemStruct, env: &ExecEnv) -> TokenStream {
    env.finish_result(run(args, item, env))
}

fn run(args: Args, item: ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let Args {
        from_crate,
        folder,
        vis,
        recursive,
    } = args;
    let vis = vis.unwrap_or_else(|| "pub".to_string());
    let vis_value: Visibility = syn::parse_str(&vis).map_err(|_| {
        env.logr.call_site_error(format!(
            "Invalid `vis` value `{vis}`, expected e.g. `pub` or `pub(crate)`"
        ))
    })?;
    let vis_str = quote! { #vis_value }.to_string();

    // Ensure we're only working with structs that have named fields
    if !matches!(item.fields, syn::Fields::Named(_)) {
        return Err(env
            .logr
            .error(item.fields.span(), "Only named struct supported"));
    }

    let syn::ItemStruct {
        attrs,
        vis,
        struct_token,
        ident,
        generics,
        fields,
        semi_token,
        ..
    } = item;
    let name_str = ident.to_string();
    let folder_str = folder.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #vis #struct_token #ident #generics #fields #semi_token

        impl #impl_generics #ident #ty_generics #where_clause {
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
                    .vis(#vis_str)
                    .recursive(#recursive)
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
        ExecEnv::builder("rsmack_fs::impls").build()
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
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("struct Levels"));
        assert!(output.contains("fn generate ()"));
        assert!(output.contains("rsmack_utils :: fs :: folder_iso_struct ()"));
        assert!(output.contains(". name (\"Levels\")"));
        assert!(output.contains(". from_crate (\"my_crate\")"));
        assert!(output.contains(". folder (\"levels\")"));
        assert!(output.contains(". vis (\"pub\")"));
        assert!(output.contains(". recursive (false)"));
        assert!(output.contains(". pre (& quote :: quote ! { # [derive (Debug)] })"));
    }

    #[test]
    fn vis_and_recursive_options_are_forwarded() {
        let args = args_from(quote! {
            from_crate = "my_crate", folder = "levels", vis = "pub(crate)", recursive = true
        });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {
                alpha: Alpha,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains(". vis (\"pub (crate)\")"));
        assert!(output.contains(". recursive (true)"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn invalid_vis_is_a_fatal_error() {
        let args = args_from(quote! {
            from_crate = "my_crate", folder = "levels", vis = "not a visibility"
        });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {
                alpha: Alpha,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Invalid `vis` value"));
    }

    #[test]
    fn tuple_struct_is_a_fatal_error() {
        let args = args_from(quote! { from_crate = "my_crate", folder = "levels" });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels(Alpha);
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Only named struct supported"));
    }
}
