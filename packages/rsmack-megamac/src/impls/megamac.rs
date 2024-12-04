use bon::Builder;
use darling::FromMeta;
use indoc::formatdoc;
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
    /// [MacroKind] as [Ident], either `Attr` or `Func`
    kind: Ident,
    /// Macro name as [Ident]
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
    let macro_impl_file_ast = get_macro_impl_file_ast(&args, &env);
    let fields_doc = get_args_fields_doc(&macro_impl_file_ast, &args, &env);
    let formatted_fields_doc = fields_doc
        .iter()
        .map(|fd| {
            format!(
                "* `{}`\n  + description: {}\n  + type: [{}]",
                fd.ident.to_string(),
                fd.doc.clone().unwrap_or("Not documented".into()),
                fd.ty.to_token_stream().to_string().replace(" ", "")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let doc_str = formatdoc!(
        "{name} procedural macro ({kind}).

        # Parameters

        {formatted_fields_doc}

        # Examples
        ",
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

fn get_macro_impl_file_ast(args: &Args, env: &ExecEnv) -> File {
    let sf_path = args.name.clone().span().span().source_file().path();
    let mut components = sf_path.components();
    components.next_back();
    let macro_impl_file_path = components
        .as_path()
        .join(env.implementations_mod_ident.clone())
        .join(format!("{}.rs", args.name.to_string()));
    let macro_impl_src = std::fs::read_to_string(macro_impl_file_path)
        .expect(&format!("Failed to get macro_impl_src of {}", args.name));
    let macro_impl_file_ast = match syn::parse_file(&macro_impl_src) {
        Ok(x)=> x,
        Err(e) => env.logr.abort_call_site(&format!(
        "Failed to parse macro_impl_src {}, this may happen for no real reason in your IDE, check that your project still build with cargo: {e:?}",
        args.name.clone()
    ))};
    macro_impl_file_ast
}
#[derive(Debug, Builder)]
struct FieldDoc {
    pub ident: Ident,
    pub doc: Option<String>,
    pub ty: Type,
}
fn get_args_fields_doc(macro_impl_file_ast: &File, args: &Args, env: &ExecEnv) -> Vec<FieldDoc> {
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
                            // TODO may not have a meta byt we should still return a FieldDoc
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
                                        .doc(lit_str.value())
                                        .ty(f.ty.clone())
                                        .build(),
                                    false => FieldDoc::builder()
                                        .ident(f.ident.clone().unwrap())
                                        .ty(f.ty.clone())
                                        .build(),
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
            "Failed to find `{}` struct in `{}` module",
            env.exec_args_ident,
            args.name.clone()
        ));
    }
}
