pub mod logr;
pub mod exec {
    #[macro_export]
    /// Call [`crate::exec::call`] on `impls` mod with `Args` args ident
    macro_rules! call_impls_with_args {
        (
            $exec_fn_mod_ident:ident,
            $item_ty:ty,
            $attr_tok_stream:ident,
            $item_tok_stream:ident
        ) => {
            exec::call!(
                impls,
                Args,
                $exec_fn_mod_ident,
                $item_ty,
                $attr_tok_stream,
                $item_tok_stream
            )
        };
    }

    /// Only for proc-macro attributes
    #[macro_export]
    macro_rules! call {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident,
        $item_ty:ty,
        $attr_tok_stream:ident,
        $item_tok_stream:ident
    ) => {{
        use ast::NestedMeta;
        use darling::*;

        let attr_args = match NestedMeta::parse_meta_list($attr_tok_stream.into()) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(Error::from(e).write_errors());
            }
        };
        let parsed_item = syn::parse_macro_input!($item_tok_stream as $item_ty);
        let parsed_args =
            match crate::$implementations_mod_ident::$exec_fn_mod_ident::$exec_args_ident::from_list(
                &attr_args,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return TokenStream::from(e.write_errors());
                }
            };
        let crate_name = std::module_path!();
        let logr = rsmack_utils::logr::Logr::builder().prefix(format!("{crate_name}::{}", stringify!($exec_fn_mod_ident))).build();
        crate::$implementations_mod_ident::$exec_fn_mod_ident::exec(
            parsed_args,
            parsed_item,
            logr
        )
        .into()
    }};
}
    pub use call;
    pub use call_impls_with_args;
}
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use bon::builder;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Ident;
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

/// Generate a struct which fields are [{folder}/{name}]: {name:pascal} with given pre struct token stream. To be called in **`build.rs`**
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
