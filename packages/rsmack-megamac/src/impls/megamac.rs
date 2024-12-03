use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::logr::Logr;
// use syn::spanned::Spanned;
use syn::*;
#[derive(Debug, FromMeta)]
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
pub fn exec(args: Args, _logr: Logr) -> TokenStream {
    let name = args.name;
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
    let imports = quote! {
        use proc_macro::TokenStream;
        use proc_macro_error2::*;
    };
    quote! {
        #imports
        #macro_impl
    }
}
