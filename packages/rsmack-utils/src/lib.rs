pub mod exec;
pub mod logr;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use bon::builder;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Ident;
/// Get the directory (workspace) from which we are compiling
pub fn calling_crate_dir() -> &'static Path {
    // Need a build.rs file
    let out_dir = Path::new(env!("OUT_DIR"));
    // TODO This work in a single crate but to check in workspace
    let calling_crate_dir = out_dir
        .parent()
        .expect("Failed to get parent of out directory")
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    calling_crate_dir
}

/// Generate a struct which fields match a flat folder of rust modules, each exposing at least a same file named **PascalCase** type
#[builder]
pub fn folder_iso_struct(
    name: &str,
    pre: TokenStream,
    folder: &str,
    // TODO find a way to remove maybe ?
    from_crate: &str,
    #[builder(default = false)] log_enabled: bool,
) -> () {
    use build_print::*;
    use stringcase::*;
    macro_rules! log {
        ($($arg:tt)+) => {
            if log_enabled {
                custom_println!("folder_iso_struct", green, $($arg)+)
            }
        };
    }

    let ccd = calling_crate_dir();
    let mods_folder_path = ccd.join(from_crate).join("src").join(folder);
    let paths = std::fs::read_dir(mods_folder_path).unwrap();
    let struct_mod_folder_id = parse_id_maybe_raw(folder);
    let fields = paths
        .into_iter()
        .filter(|path| {
            let module_path = path.as_ref().unwrap().path();
            let struct_folder_file_stem = module_path.file_stem().unwrap().to_str().unwrap();
            struct_folder_file_stem != "mod"
        })
        .map(|path| {
            let module_path = path.unwrap().path();
            let struct_folder_file_stem = module_path.file_stem().unwrap().to_str().unwrap();
            let field_ty_name = pascal_case_with_sep(struct_folder_file_stem, "_");
            let struct_mod_id = parse_id_maybe_raw(struct_folder_file_stem);
            let import_path = quote! {crate::#struct_mod_folder_id::#struct_mod_id};
            let field_ty_id = parse_id_maybe_raw(&field_ty_name);
            let field_ty_path = quote! {
                #import_path::#field_ty_id
            };
            log!("{} -> {}", name, field_ty_path.to_string());
            quote! {
                #struct_mod_id: #field_ty_path
            }
        })
        .collect::<Vec<_>>();
    let name_id = parse_id_maybe_raw(name);
    let output = quote! {
        #pre
        pub struct #name_id {
            #(#fields),*
        }
    }
    .to_token_stream()
    .to_string();
    generate_file(format!("{name}.rs"), output.as_bytes());
}

fn parse_id_maybe_raw(s: &str) -> Ident {
    let id = syn::parse_str::<Ident>(&s).unwrap_or_else(|_| Ident::new_raw(&s, Span::call_site()));
    id
}

fn generate_file<P: AsRef<Path>>(path: P, text: &[u8]) {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let dest_path = out_path.join(&path);
    use std::io::Write;
    let mut f = File::create(dest_path).unwrap();
    f.write_all(text).unwrap()
}
