use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::calling_crate_dir, megamac::ExecEnv};
use strum::Display;
use syn::{spanned::Spanned, *};
#[derive(Debug, PartialEq, Display, FromMeta)]
enum MacroKind {
    Func,
    Attr,
}
#[derive(Debug, FromMeta)]
pub struct Args {
    kind: Ident,
    name: Ident,
}

impl From<Ident> for MacroKind {
    fn from(value: Ident) -> Self {
        match value.to_string().as_str() {
            "Attr" => Self::Attr,
            "Func" => Self::Func,
            _ => panic!("Unsupported MacroKind"),
        }
    }
}
/// Execute megamac macro
pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    let name = args.name.clone();
    let imports = quote! {
        use proc_macro::TokenStream;
        use proc_macro_error2::*;
    };
    let kind = args.kind.to_string();
    let sf_path = name.span().span().source_file().path();
    let mut components = sf_path.components();
    components.next_back();
    let macro_impl_file_path = components
        .as_path()
        .join(env.implementations_mod_ident)
        .join(format!("{}.rs", args.name.to_string()));
    let macro_impl_src =
        std::fs::read_to_string(macro_impl_file_path).expect("Failed to get macro_impl_src");
    let macro_impl_file_ast = syn::parse_file(&macro_impl_src).expect(&format!(
        "Failed to parse macro_impl_src {}",
        args.name.clone()
    ));
    let args_item = macro_impl_file_ast.items.iter().find(|i| match i {
        Item::Struct(ItemStruct { ident, .. }) => ident.to_string() == env.exec_args_ident,
        _ => false,
    });
    if let Some(args_item) = args_item {
        env.logr.abort_call_site(&format!("{args_item:#?}",));
    }
    let args_link = format!("{name}::Args");
    let doc_str = format!(
        "{} procedural macro ({}). See [`Args`]({args_link})",
        name, kind
    );
    let macro_impl = match args.kind.into() {
        MacroKind::Func => quote! {
            #[proc_macro_error]
            #[proc_macro]
            pub fn #name(args: TokenStream) -> TokenStream {
                rsmack_utils::exec::call_func_impls_with_args!(#name, args)
            }
        },
        MacroKind::Attr => quote! {
            #[proc_macro_error]
            #[proc_macro_attribute]
            pub fn #name(attr: TokenStream, item: TokenStream) -> TokenStream {
                rsmack_utils::exec::call_attr_impls_with_args!(#name, syn::ItemStruct, attr, item)
            }
        },
    };

    quote! {
        #imports

        #[doc = #doc_str]
        #macro_impl
    }
}
