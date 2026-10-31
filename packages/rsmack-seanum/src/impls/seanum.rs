use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;

#[derive(Debug, FromMeta)]
pub struct Args {
    /// The Rust type for the enum in the database
    rs_type: syn::Ident,
    /// The database type for the enum
    db_type: syn::LitStr,
}

/// Execute seanum macro
pub fn exec(args: Args, item: syn::ItemEnum, env: ExecEnv) -> TokenStream {
    env.finish(run(args, item, &env))
}

fn run(args: Args, item: syn::ItemEnum, _env: &ExecEnv) -> TokenStream {
    let enum_name = &item.ident;
    // Convert enum name to snake_case for the database name
    let db_enum_name = stringcase::snake_case(&enum_name.to_string());

    // Collect existing attributes (excluding the seanum attribute)
    let existing_attrs: Vec<_> = item
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("seanum"))
        .collect();

    // Generate the string values for each variant
    let variants_with_attrs: Vec<_> = item
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_name_str = variant_name.to_string();
            let existing_variant_attrs = &variant.attrs;
            let sea_orm_attr = quote! {
                #[sea_orm(string_value = #variant_name_str)]
            };

            quote! {
                #(#existing_variant_attrs)*
                #sea_orm_attr
                #variant_name
            }
        })
        .collect();

    let visibility = &item.vis;
    let generics = &item.generics;
    let rs_type = args.rs_type.to_string();
    let db_type = args.db_type;

    quote! {
        #(#existing_attrs)*
        #[derive(
            Clone, ::fake::Dummy, Debug, PartialEq, ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum, Eq, ::serde::Serialize, ::serde::Deserialize, Hash
        )]
        #[sea_orm(rs_type = #rs_type, db_type = #db_type, enum_name = #db_enum_name)]
        #visibility enum #enum_name #generics {
            #(#variants_with_attrs),*
        }
    }
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
        ExecEnv::builder("rsmack_seanum::impls", "impls", "Args", "seanum").build()
    }

    #[test]
    fn generates_sea_orm_enum_with_snake_case_name() {
        let args = args_from(quote! { rs_type = "String", db_type = "String" });
        let item: syn::ItemEnum = syn::parse_quote! {
            #[derive(Default)]
            enum UserStatus {
                #[default]
                Active,
                Suspended,
            }
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("enum_name = \"user_status\""));
        assert!(output.contains("rs_type = \"String\""));
        assert!(output.contains("db_type = \"String\""));
        assert!(output.contains("# [sea_orm (string_value = \"Active\")]"));
        assert!(output.contains("# [sea_orm (string_value = \"Suspended\")]"));
        assert!(output.contains("DeriveActiveEnum"));
        assert!(!output.contains("seanum"));
        // existing attributes are preserved
        assert!(output.contains("derive (Default)"));
        // the #[default] variant attribute is preserved, followed by the generated attr
        assert!(output.contains("# [default] # [sea_orm (string_value = \"Active\")] Active"));
    }

    #[test]
    fn empty_enum_is_supported() {
        let args = args_from(quote! { rs_type = "i32", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Empty {}
        };
        let output = exec(args, item, env()).to_string();
        assert!(output.contains("enum_name = \"empty\""));
        assert!(output.contains("rs_type = \"i32\""));
        assert!(output.contains("db_type = \"Integer\""));
        assert!(output.contains("enum Empty"));
        assert!(!output.contains("string_value"));
    }
}
