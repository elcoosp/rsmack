use bon::Builder;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use strum::Display;
use syn::*;
#[derive(Debug, PartialEq, Display, FromMeta)]
enum MacroKind {
    Func,
    Attr,
    Derive,
}
#[derive(Debug, FromMeta)]
pub struct Args {
    /// [MacroKind] as [Ident], either `Attr` or `Func`
    kind: Ident,

    /// Macro name as [Ident]
    name: Ident,

    #[darling(default)]
    /// The receiver [syn] type of the macro attr, only for [MacroKind::Attr]
    receiver: Option<Ident>,
}

impl From<Ident> for MacroKind {
    fn from(value: Ident) -> Self {
        match value.to_string().as_str() {
            "Attr" => Self::Attr,
            "Func" => Self::Func,
            "Derive" => Self::Derive,
            _ => panic!("Unsupported MacroKind"),
        }
    }
}
// FIXME Does not work correctly on multi line comments because this is maybe in ast many meta items, hence the flat_map may not be valid
/// Execute megamac macro
pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    let name = args.name.clone();
    let receiver = args.receiver.clone();
    let imports = quote! {
        use proc_macro_error2::*;
    };
    let kind = args.kind.to_string();
    let macro_impl_file_ast = get_macro_impl_file_ast(&args, &env);
    let fields_doc = get_args_fields_doc(&macro_impl_file_ast, &args, &env);
    let get_arg_field_const_id = |fd: &FieldDoc| {
        Ident::new(
            &format!(
                "{}_ARGS_FIELD_TYPE_QUALIFIED_PATH_{}",
                name.to_string().to_uppercase(),
                fd.ident.to_string().to_uppercase()
            ),
            fd.ident.span(),
        )
    };
    let formatted_fields_doc = fields_doc
        .iter()
        .map(|fd| {
            let template_without_ty_qualified_path = format!(
                "* `{}` - {}\n  + type: [`{}`]",
                fd.ident.to_string(),
                fd.doc.clone().unwrap_or("Not documented".into()),
                fd.ty.to_token_stream().to_string().replace(" ", ""),
            );

            // FIXME get_arg_field_const_id is useless since doc can just accept a literal, maybe create a macro edoc ?
            // let const_id = get_arg_field_const_id(fd);
            quote! { #[doc = #template_without_ty_qualified_path]}
        })
        .collect::<Vec<_>>();
    let name_str = name.to_string();
    let kind_str = kind.to_string();
    let doc = quote! {
        #[doc = concat!(#name_str, " procedural macro (", #kind_str, ").")]
        #[doc ="# Parameters"]
        #(#formatted_fields_doc)*
        #[doc ="# Examples"]
    };
    let macro_impl = match args.kind.into() {
        MacroKind::Derive => {
            let derive_name = Ident::new(&stringcase::pascal_case(&name_str), name.span());
            quote! {
                #[proc_macro_error]
                #[proc_macro_derive(#derive_name, attributes(#name))]
                pub fn #name(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    rsmack_utils::exec::call_derive_impls_with_args!(#name, item)
                }
            }
        }
        MacroKind::Func => quote! {
            #[proc_macro_error]
            #[proc_macro]
            pub fn #name(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
                rsmack_utils::exec::call_func_impls_with_args!(#name, args)
            }
        },
        MacroKind::Attr => quote! {
            #[proc_macro_error]
            #[proc_macro_attribute]
            pub fn #name(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                rsmack_utils::exec::call_attr_impls_with_args!(#name, syn::#receiver, attr, item)
            }
        },
    };

    let consts = fields_doc.iter().map(|fd| {
        let const_id = get_arg_field_const_id(fd);
        let ty = fd.ty.clone();
        quote! {
            static #const_id:&'static str = std::any::type_name::<#ty>();

        }
    });
    quote! {
        #imports
        // #(#consts)*
        #doc
        #macro_impl
    }
}

fn get_macro_impl_file_ast(args: &Args, env: &ExecEnv) -> File {
    let package_src_folder = package_src_folder();
    let macro_impl_file_path = package_src_folder
        .join(env.implementations_mod_ident.clone())
        .join(format!("{}.rs", args.name.to_string()));
    let macro_impl_src = std::fs::read_to_string(macro_impl_file_path.clone()).expect(&format!(
        "Failed to get macro_impl_src of {} at {macro_impl_file_path:?}",
        args.name
    ));
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
