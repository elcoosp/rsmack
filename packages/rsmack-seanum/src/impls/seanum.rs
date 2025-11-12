use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::*;

#[derive(Debug, FromMeta)]
pub struct Args {
    /// The Rust type for the enum in the database
    rs_type: syn::Ident,
    /// The database type for the enum
    db_type: syn::LitStr,
}

/// Execute seanum macro
pub fn exec(args: Args, item: ItemEnum, _env: ExecEnv) -> TokenStream {
    let enum_name = &item.ident;
    let enum_name_str = enum_name.to_string();

    // Convert enum name to snake_case for database name
    let db_enum_name = enum_name_str
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && c.is_uppercase() {
                format!("_{}", c.to_lowercase().next().unwrap())
            } else {
                c.to_lowercase().next().unwrap().to_string()
            }
        })
        .collect::<String>();

    // Generate the string values for each variant
    let variants_with_attrs: Vec<_> = item
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_name_str = variant_name.to_string();

            // Create the sea_orm attribute for this variant
            let sea_orm_attr = quote! {
                #[sea_orm(string_value = #variant_name_str)]
            };

            quote! {
                #sea_orm_attr
                #variant_name
            }
        })
        .collect();

    // Generate the final enum
    let visibility = &item.vis;
    let generics = &item.generics;
    let rs_type = args.rs_type.to_string();
    let db_type = args.db_type;
    let result = quote! {
        use fake::Dummy;
        use sea_orm::entity::prelude::*;
        use sea_orm_migration::prelude::*;
        use serde::{Deserialize, Serialize};

        #[derive(
            Clone, Dummy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Eq, Serialize, Deserialize,
        )]
        #[sea_orm(rs_type = #rs_type, db_type = #db_type, enum_name = #db_enum_name)]
        #visibility enum #enum_name #generics {
            #(#variants_with_attrs),*
        }
    };

    result
}
