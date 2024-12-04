use bon::Builder;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::megamac::ExecEnv;
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
        .join(env.implementations_mod_ident.clone())
        .join(format!("{}.rs", args.name.to_string()));
    let macro_impl_src =
        std::fs::read_to_string(macro_impl_file_path).expect("Failed to get macro_impl_src");
    let macro_impl_file_ast = syn::parse_file(&macro_impl_src).expect(&format!(
        "Failed to parse macro_impl_src {}",
        args.name.clone()
    ));
    let fields_doc = get_args_fields_doc(&macro_impl_file_ast, &env, &args);
    let args_link = format!("{name}::Args");
    // env.logr.abort_call_site(&format!("{fields_doc:#?}"));
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
#[derive(Debug, Builder)]
struct FieldDoc {
    pub ident: Ident,
    pub doc: Option<String>,
}
fn get_args_fields_doc(macro_impl_file_ast: &File, env: &ExecEnv, args: &Args) -> Vec<FieldDoc> {
    let args_item = macro_impl_file_ast.items.iter().find(|i| match i {
        Item::Struct(ItemStruct { ident, .. }) => ident.to_string() == env.exec_args_ident,
        _ => false,
    });

    if let Some(args_item) = args_item {
        let fields_doc = match args_item {
            Item::Struct(ItemStruct { fields, .. }) => fields
                .iter()
                .flat_map(|f| {
                    f.attrs
                        .iter()
                        .map(|a| match a.meta.clone() {
                            Meta::NameValue(MetaNameValue {
                                value:
                                    Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit_str),
                                        ..
                                    }),
                                path: Path { segments, .. },
                                ..
                            }) => match segments.first().unwrap() {
                                PathSegment { ident, .. } => match ident.to_string() == "doc" {
                                    true => FieldDoc::builder()
                                        .ident(f.ident.clone().unwrap())
                                        .doc(lit_str.to_token_stream().to_string())
                                        .build(),
                                    false => {
                                        FieldDoc::builder().ident(f.ident.clone().unwrap()).build()
                                    }
                                },
                            },
                            _ => unimplemented!(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        };
        fields_doc
    } else {
        env.logr.abort_call_site(&format!(
            "Failed to find `Args` struct in `{}` module",
            args.name.clone()
        ));
    }
}
