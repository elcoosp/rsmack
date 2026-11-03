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

/// Per-field configuration parsed from `#[wrap(...)]` attributes.
#[derive(Debug, Default, FromMeta)]
struct FieldArgs {
    /// Skip the field entirely (`#[wrap(skip)]`)
    #[darling(default)]
    skip: bool,
    /// Per-field wrapper override (`#[wrap(with = "Rc")]`)
    #[darling(default)]
    with: Option<syn::Ident>,
    /// Replace the inner type instead of wrapping it (`#[wrap(with = "Vec", map = "u8")]`
    /// produces `Vec<u8>` regardless of the field type)
    #[darling(default)]
    map: Option<syn::Type>,
}

/// Execute wrap macro, **before serde derive, otherwise it will error**
pub fn exec(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> TokenStream {
    env.finish_result(run(args, item, env))
}

fn run(args: Args, item: syn::ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let Args { with } = args;
    let mut transformed_item = item;
    transformed_item.fields = match transformed_item.fields {
        syn::Fields::Named(fields_named) => {
            let transformed_fields_named = fields_named
                .named
                .into_iter()
                .map(|f| wrap_field(&with, f, env))
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

/// Wrap the inner type of a [`syn::Field`] according to its `#[wrap(...)]` config.
///
/// Every field type can be wrapped (`Option<&'static str>`, `Box<dyn Trait>`,
/// `Rc<fn(u8) -> bool>`, ... are all legal Rust); the consumed `#[wrap(...)]` attribute
/// is stripped so the emitted struct stays valid.
fn wrap_field(default_wrapper: &syn::Ident, mut f: syn::Field, env: &ExecEnv) -> syn::Field {
    let mut config = FieldArgs::default();
    f.attrs.retain_mut(|attr| {
        if !attr.path().is_ident("wrap") {
            return true;
        }
        match FieldArgs::from_meta(&attr.meta) {
            Ok(parsed) => {
                config = parsed;
                false
            }
            Err(e) => {
                env.logr
                    .emit_error(attr.span(), format!("Failed to parse wrap attribute: {e}"));
                false
            }
        }
    });
    if config.skip {
        return f;
    }
    let wrapper = config.with.unwrap_or_else(|| default_wrapper.clone());
    // `map` replaces the inner type entirely: `with = "Vec", map = "u8"` -> `Vec<u8>`;
    // otherwise the field type is wrapped as-is (every type wraps legally).
    let inner = config.map.unwrap_or_else(|| f.ty.clone());
    f.ty = syn::parse2(quote! { #wrapper<#inner> })
        .expect("wrapper type tokens are always a valid type");
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    fn env() -> ExecEnv {
        ExecEnv::new("rsmack_wrap::impls")
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
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("name : Option < String >"));
        assert!(output.contains("tags : Option < Vec < String > >"));
        assert!(output.contains("point : Option < (u8 , u8) >"));
        assert!(output.contains("nums : Option < [u8 ; 2] >"));
        assert!(!output.contains("compile_error"));
    }

    /// v2: previously "unsupported" types (references, bare fns, trait objects, raw
    /// pointers) wrap perfectly legally — the restriction is lifted.
    #[test]
    fn wraps_previously_unsupported_types() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Demo {
                reference: &'static str,
                callback: fn(u8) -> bool,
                object: Box<dyn Send>,
                pointer: *const u8,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("reference : Option < & 'static str >"));
        assert!(output.contains("callback : Option < fn (u8) -> bool >"));
        assert!(output.contains("object : Option < Box < dyn Send > >"));
        assert!(output.contains("pointer : Option < * const u8 >"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn tuple_struct_is_a_fatal_error() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Demo(String);
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Only named struct supported"));
    }

    #[test]
    fn skip_attribute_leaves_the_field_untouched() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct User {
                name: String,
                #[wrap(skip)]
                untouched: &'static str,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("name : Option < String >"));
        assert!(output.contains("untouched : & 'static str"));
        // the consumed attribute is stripped from the output
        assert!(!output.contains("wrap (skip)"));
        assert!(!output.contains("wrap ("));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn per_field_with_overrides_the_default_wrapper() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct User {
                name: String,
                #[wrap(with = "Box")]
                boxed: String,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("name : Option < String >"));
        assert!(output.contains("boxed : Box < String >"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn map_replaces_the_inner_type() {
        let args = args_from(quote! { with = "Vec" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Event {
                #[wrap(with = "Vec", map = "u8")]
                raw: String,
                plain: String,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("raw : Vec < u8 >"));
        assert!(output.contains("plain : Vec < String >"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn broken_wrap_attribute_is_collected_and_stripped() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct User {
                #[wrap(unknown_arg = 12)]
                name: String,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to parse wrap attribute"));
        assert!(!output.contains("wrap ("));
    }

    #[test]
    fn non_wrap_attributes_are_preserved() {
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct User {
                #[serde(rename = "n")]
                name: String,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("serde (rename"));
        assert!(output.contains("name : Option < String >"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn skip_alone_works_without_wrapper() {
        // A field only marked `skip` never hits the wrapper code path
        let args = args_from(quote! { with = "Option" });
        let item: syn::ItemStruct = syn::parse_quote! {
            struct Demo {
                #[wrap(skip)]
                reference: &'static str,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("reference : & 'static str"));
        assert!(!output.contains("compile_error"));
    }
}
