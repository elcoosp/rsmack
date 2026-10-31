use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;

#[derive(Debug, FromMeta)]
pub struct Args {
    /// The type with which to wrap the struct item fields
    with: syn::Ident,
}

/// Execute wrap macro, **before serde derive, otherwise it will error**
pub fn exec(args: Args, item: syn::ItemStruct, env: ExecEnv) -> TokenStream {
    match run(args, item, &env) {
        Ok(output) => env.finish(output),
        Err(error) => env.finish(error.to_compile_error()),
    }
}

fn run(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let mut transformed_item = item;
    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => {
            let transformed_fields_named = fields_named
                .named
                .into_iter()
                .map(|f| wrap_field(&args.with, f, env))
                .collect();
            syn::Fields::Named(syn::FieldsNamed {
                brace_token: fields_named.brace_token,
                named: transformed_fields_named,
            })
        }
        fields => return Err(env.logr.error(fields.span(), "Only named struct supported")),
    };
    Ok(quote! {
        #transformed_item
    })
}

/// Wrap the inner type of a [`syn::Field`] in a generic arguments call of `wrapper`
///
/// Unsupported field types emit a non-fatal error and are left untouched.
fn wrap_field(wrapper: &syn::Ident, f: syn::Field, env: &ExecEnv) -> syn::Field {
    let mut f = f;
    let ty = f.ty;
    match &ty {
        syn::Type::Path(_) | syn::Type::Slice(_) | syn::Type::Tuple(_) | syn::Type::Array(_) => {
            f.ty = syn::parse2(quote! { #wrapper<#ty> })
                .expect("wrapper type tokens are always a valid type");
        }
        _ => {
            let kind = format!("{ty:?}")
                .split(['(', ' '])
                .next()
                .unwrap_or_default()
                .trim_start_matches("Type::")
                .to_string();
            env.logr.emit_error(
                ty.span(),
                format!("Field type not supported `{kind}`, cannot wrap it"),
            );
            f.ty = ty;
        }
    }
    f
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
        ExecEnv::builder("rsmack_wrap::impls", "impls", "Args", "wrap").build()
    }

    #[test]
    fn wraps_every_supported_field_type() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct User {
                name: String,
                tags: Vec<String>,
                point: (u8, u8),
                nums: [u8; 2],
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("name : Option < String >"));
        assert!(output.contains("tags : Option < Vec < String > >"));
        assert!(output.contains("point : Option < (u8 , u8) >"));
        assert!(output.contains("nums : Option < [u8 ; 2] >"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn unsupported_field_type_emits_collected_error_and_keeps_field() {
        let args = args_from(quote! { with = "Box" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Demo {
                name: String,
                reference: &'static str,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("name : Box < String >"));
        // the unsupported field is kept as-is, with a compile_error appended
        assert!(output.contains("reference : & 'static str"));
        assert!(output.contains("compile_error"));
        assert!(
            output.contains("Field type not supported `Reference`"),
            "unexpected output: {output}"
        );
    }

    #[test]
    fn tuple_struct_is_a_fatal_error() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Demo(String);
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Only named struct supported"));
    }
}
