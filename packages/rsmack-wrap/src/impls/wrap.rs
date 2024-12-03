use darling::FromMeta;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::{quote, ToTokens};
use rsmack_utils::logr::Logr;
use syn::spanned::Spanned;
use syn::*;
#[derive(Debug, FromMeta)]
pub struct Args {
    with: Ident,
    // use syn::punctuated::Punctuated;
    // derive: Option<Punctuated<syn::Ident, Token![,]>>,
}
/// Execute wrap macro, **before serde derive, otherwise it will error**
pub fn exec(args: Args, item: ItemStruct, logr: Logr) -> TokenStream {
    let mut transformed_item = item.clone();
    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => {
            let transformed_fields_named = fields_named
                .named
                .into_iter()
                .map(|f| {
                    let wrapper_ty = args.with.clone();
                    let ty = f.ty.clone();
                    match ty {
                        Type::Path(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Slice(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Tuple(t) => wrap_field_ty(wrapper_ty, t, f),
                        Type::Array(t) => wrap_field_ty(wrapper_ty, t, f),
                        _ => {
                            let ty = f.ty.clone();
                            let ty_span = ty.span().unwrap();
                            emit_error!(
                                ty_span.clone(),
                                format!(
                                    "#[rsmack_wrap::wrap]: Field type not supported {}",
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
                named: transformed_fields_named,
            })
        }
        _ => logr.abort(
            transformed_item.fields.span(),
            "Only named struct supported",
        ),
    };
    quote! {
        #transformed_item
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
