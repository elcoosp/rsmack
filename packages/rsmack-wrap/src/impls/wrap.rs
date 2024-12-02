use darling::FromMeta;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::*;
#[derive(Debug, FromMeta)]
pub struct Args {
    wrapper: Ident,
    // use syn::punctuated::Punctuated;
    // derive: Option<Punctuated<syn::Ident, Token![,]>>,
}
/// Execute wrap macro
pub fn exec(args: Args, item: ItemStruct) -> TokenStream {
    let mut optioned_item = item.clone();
    optioned_item.fields = match optioned_item.fields {
        syn::Fields::Named(fields_named) => {
            let opt_fields_named = fields_named
                .named
                .into_iter()
                .map(|f| {
                    let wrapper_ty = args.wrapper.clone();
                    let ty = f.ty.clone();
                    let ty_span = ty.span().unwrap();
                    match ty {
                        Type::Path(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Slice(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Tuple(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Array(t) => wrap_field_ty(wrapper_ty, t, f),
                        _ => {
                            let ty = f.ty.clone();
                            emit_error!(
                                ty_span.clone(),
                                format!(
                                    "#[opt_macros::wrap]: Field type not supported {}",
                                    format!("{:?}", ty).split(" ").next().unwrap()
                                )
                            );
                            Field {
                                ty: ty.clone(),
                                attrs: f.attrs.clone(),
                                vis: f.vis.clone(),
                                mutability: f.mutability.clone(),
                                ident: f.ident.clone(),
                                colon_token: f.colon_token.clone(),
                            }
                        }
                    }
                })
                .collect();
            syn::Fields::Named(FieldsNamed {
                brace_token: fields_named.brace_token.clone(),
                named: opt_fields_named,
            })
        }
        _ => abort!(optioned_item.fields.span(), "Only named struct supported"),
    };
    quote! {
        #optioned_item
    }
}
/// Wrap the inner type of a [Field] in a type generic arguments call
fn wrap_field_ty(wrapper: impl ToTokens, inner_ty: impl ToTokens, f: Field) -> Field {
    let wrapper_token_stream = wrapper.to_token_stream();
    let type_path_token_stream = inner_ty.to_token_stream();
    let ty: Type =
        syn::parse_str(&format!("{wrapper_token_stream}<{type_path_token_stream}>")).unwrap();
    Field {
        ty,
        attrs: f.attrs.clone(),
        vis: f.vis.clone(),
        mutability: f.mutability.clone(),
        ident: f.ident.clone(),
        colon_token: f.colon_token.clone(),
    }
}
