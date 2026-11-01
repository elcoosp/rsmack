use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;

/// Integer Rust types supported as `rs_type` with automatic `TryFrom` impls.
const INTEGER_RS_TYPES: [&str; 10] = [
    "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize",
];

#[derive(Debug, FromMeta)]
pub struct Args {
    /// The Rust type for the enum in the database
    rs_type: syn::Ident,
    /// The database type for the enum
    db_type: syn::LitStr,
    /// Extra derives appended to the generated derive list (`derives("Display", "Eq")`)
    #[darling(default)]
    derives: Vec<syn::LitStr>,
    /// Generate the `sea-orm-migration` `ColumnDef` helper (`migration = true`)
    #[darling(default)]
    migration: bool,
}

/// Per-variant configuration parsed from `#[seanum(...)]` attributes.
#[derive(Debug, Default, FromMeta)]
struct VariantArgs {
    /// Database value override (`#[seanum(rename = "active")]`)
    #[darling(default)]
    rename: Option<syn::LitStr>,
}

/// Execute seanum macro
pub fn exec(args: Args, item: syn::ItemEnum, env: &ExecEnv) -> TokenStream {
    env.finish_result(run(args, item, env))
}

fn run(args: Args, item: syn::ItemEnum, env: &ExecEnv) -> syn::Result<TokenStream> {
    let syn::ItemEnum {
        attrs,
        vis,
        ident: enum_name,
        generics,
        variants,
        ..
    } = item;
    // Convert enum name to snake_case for the database name
    let db_enum_name = stringcase::snake_case(&enum_name.to_string());
    let integer_rs_type = INTEGER_RS_TYPES.contains(&args.rs_type.to_string().as_str());

    // Collect existing attributes (excluding the seanum attribute)
    let existing_attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("seanum"))
        .collect();

    // Generate the string (or integer) value for each variant
    let mut variants_with_attrs: Vec<TokenStream> = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        let variant_name = &variant.ident;
        let mut rename: Option<syn::LitStr> = None;
        let mut existing_variant_attrs: Vec<&syn::Attribute> = Vec::new();
        for attr in &variant.attrs {
            if attr.path().is_ident("seanum") {
                match VariantArgs::from_meta(&attr.meta) {
                    Ok(parsed) => rename = parsed.rename,
                    Err(e) => env.logr.emit_error(
                        attr.span(),
                        format!("Failed to parse seanum variant attribute: {e}"),
                    ),
                }
            } else {
                existing_variant_attrs.push(attr);
            }
        }
        let value = rename
            .unwrap_or_else(|| syn::LitStr::new(&variant_name.to_string(), variant_name.span()));
        let index_lit = syn::LitInt::new(&index.to_string(), variant_name.span());
        let sea_orm_attr = if integer_rs_type {
            quote! { #[sea_orm(num_value = #index_lit)] }
        } else {
            quote! { #[sea_orm(string_value = #value)] }
        };

        variants_with_attrs.push(quote! {
            #(#existing_variant_attrs)*
            #sea_orm_attr
            #variant_name
        });
    }

    let extra_derives: Vec<syn::Path> = args
        .derives
        .iter()
        .map(|derive| syn::parse_str::<syn::Path>(&derive.value()))
        .collect::<syn::Result<_>>()
        .map_err(|e| {
            env.logr
                .error(args.rs_type.span(), format!("Invalid `derives` entry: {e}"))
        })?;
    let rs_type = args.rs_type.to_string();
    let rs_type_ident = &args.rs_type;
    let db_type = args.db_type;

    // Integer `rs_type` gets a `#[repr]` plus an index-based `TryFrom` impl.
    let repr_attr = integer_rs_type.then(|| quote! { #[repr(#rs_type_ident)] });
    let try_from_impl = if integer_rs_type {
        generate_try_from(&enum_name, &variants, &args.rs_type, env)
    } else {
        quote! {}
    };

    let migration_impl = if args.migration {
        generate_migration_column_def(&enum_name, &db_enum_name, &db_type.value(), env)
    } else {
        quote! {}
    };

    Ok(quote! {
        #(#existing_attrs)*
        #[derive(
            Clone, ::fake::Dummy, Debug, PartialEq, ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum, Eq, ::serde::Serialize, ::serde::Deserialize, Hash, #(#extra_derives),*
        )]
        #[sea_orm(rs_type = #rs_type, db_type = #db_type, enum_name = #db_enum_name)]
        #repr_attr
        #vis enum #enum_name #generics {
            #(#variants_with_attrs),*
        }

        #try_from_impl

        #migration_impl
    })
}

/// Generate an index-based `TryFrom<rs_type>` impl for an integer-backed enum.
///
/// Requires unit variants without explicit discriminants; when violated, a non-fatal
/// error is emitted and no impl is generated.
fn generate_try_from(
    enum_name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    rs_type: &syn::Ident,
    env: &ExecEnv,
) -> TokenStream {
    for variant in variants {
        if !variant.fields.is_empty() {
            env.logr.emit_error(
                variant.span(),
                format!(
                    "Failed to generate TryFrom: variant `{}` has fields, only unit variants are supported with integer `rs_type`",
                    variant.ident
                ),
            );
            return quote! {};
        }
        if variant.discriminant.is_some() {
            env.logr.emit_error(
                variant.span(),
                format!(
                    "Failed to generate TryFrom: variant `{}` has an explicit discriminant, implicit index values are required with integer `rs_type`",
                    variant.ident
                ),
            );
            return quote! {};
        }
    }
    let arms = variants.iter().enumerate().map(|(index, variant)| {
        let ident = &variant.ident;
        let index_lit = syn::LitInt::new(&index.to_string(), variant.ident.span());
        quote! { #index_lit => ::core::result::Result::Ok(Self::#ident), }
    });
    quote! {
        impl ::core::convert::TryFrom<#rs_type> for #enum_name {
            type Error = ::sea_orm::DbErr;

            fn try_from(value: #rs_type) -> ::core::result::Result<Self, Self::Error> {
                match value {
                    #(#arms)*
                    other => ::core::result::Result::Err(::sea_orm::DbErr::Custom(format!(
                        "Invalid integer value for {}: {}",
                        stringify!(#enum_name),
                        other
                    ))),
                }
            }
        }
    }
}

/// `db_type` to zero-argument `sea_query::ColumnDef` method mapping.
const COLUMN_DEF_METHODS: [(&str, &str); 17] = [
    ("String", "string"),
    ("Text", "text"),
    ("TinyInteger", "tiny_integer"),
    ("SmallInteger", "small_integer"),
    ("Integer", "integer"),
    ("BigInt", "big_integer"),
    ("TinyUnsigned", "tiny_unsigned"),
    ("SmallUnsigned", "small_unsigned"),
    ("Unsigned", "unsigned"),
    ("BigUnsigned", "big_unsigned"),
    ("Float", "float"),
    ("Double", "double"),
    ("Boolean", "boolean"),
    ("Date", "date"),
    ("DateTime", "date_time"),
    ("Json", "json"),
    ("JsonBinary", "json_binary"),
];

/// Generate the `sea-orm-migration` helper returning a ready-made `ColumnDef` for the
/// enum's column, removing boilerplate from migration files.
fn generate_migration_column_def(
    enum_name: &syn::Ident,
    db_enum_name: &str,
    db_type: &str,
    env: &ExecEnv,
) -> TokenStream {
    let Some((_, method)) = COLUMN_DEF_METHODS
        .iter()
        .find(|(supported, _)| *supported == db_type)
    else {
        env.logr.emit_error(
            enum_name.span(),
            format!(
                "Failed to generate the migration helper: db_type `{db_type}` has no `ColumnDef` mapping"
            ),
        );
        return quote! {};
    };
    let method = quote::format_ident!("{method}");
    quote! {
        impl #enum_name {
            /// `sea-orm-migration` helper: a `ColumnDef` for this enum's column, ready
            /// to be passed to `.col(...)` inside a migration.
            #[must_use]
            pub fn migration_column_def() -> ::sea_orm_migration::sea_query::ColumnDef {
                ::sea_orm_migration::sea_query::ColumnDef::new(
                    ::sea_orm_migration::sea_query::Alias::new(#db_enum_name)
                )
                .#method()
            }
        }
    }
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
        ExecEnv::builder("rsmack_seanum::impls").build()
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
        let output = exec(args, item, &env()).to_string();
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
        // no TryFrom impl for string rs_type
        assert!(!output.contains("convert :: TryFrom"));
    }

    #[test]
    fn empty_enum_is_supported() {
        let args = args_from(quote! { rs_type = "i32", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Empty {}
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("enum_name = \"empty\""));
        assert!(output.contains("rs_type = \"i32\""));
        assert!(output.contains("db_type = \"Integer\""));
        assert!(output.contains("enum Empty"));
        assert!(!output.contains("string_value"));
    }

    #[test]
    fn extra_derives_are_appended() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", derives("Display", "Eq")
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Level { Low, High }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("Hash , Display , Eq)"));
    }

    #[test]
    fn invalid_derives_entry_is_a_prefixed_error() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", derives("not a path !!!")
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Level { Low }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Invalid `derives` entry"));
    }

    #[test]
    fn variant_rename_overrides_the_database_value() {
        let args = args_from(quote! { rs_type = "String", db_type = "String" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus {
                #[seanum(rename = "ACTIVE")]
                Active,
                #[seanum(unknown = 12)]
                Suspended,
            }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("# [sea_orm (string_value = \"ACTIVE\")] Active"));
        // renamed and unknown attributes are stripped from the output
        assert!(!output.contains("seanum ("));
        // the unknown variant attribute emitted a collected error
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to parse seanum variant attribute"));
    }

    #[test]
    fn integer_rs_type_generates_repr_num_values_and_try_from() {
        let args = args_from(quote! { rs_type = "i32", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Priority { Low, Medium, High }
        };
        let output = exec(args, item, &env()).to_string();
        eprintln!("DEBUG OUTPUT: {output}");
        assert!(output.contains("# [repr (i32)]"));
        assert!(output.contains("# [sea_orm (num_value = 0)] Low"));
        assert!(output.contains("# [sea_orm (num_value = 1)] Medium"));
        assert!(output.contains("# [sea_orm (num_value = 2)] High"));
        assert!(output.contains("impl :: core :: convert :: TryFrom < i32 > for Priority"));
        assert!(output.contains("0 => :: core :: result :: Result :: Ok (Self :: Low)"));
        assert!(output.contains("Invalid integer value for {}"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn integer_rs_type_with_non_unit_variant_skips_try_from() {
        let args = args_from(quote! { rs_type = "u8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Weird { Plain(u8), Tagged }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("# [repr (u8)]"));
        assert!(!output.contains("convert :: TryFrom"));
        assert!(output.contains("compile_error"));
        assert!(output.contains("only unit variants are supported"));
    }

    #[test]
    fn integer_rs_type_with_discriminant_skips_try_from() {
        let args = args_from(quote! { rs_type = "u8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Flags { A = 1, B = 2 }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(!output.contains("convert :: TryFrom"));
        assert!(output.contains("explicit discriminant"));
    }

    #[test]
    fn migration_helper_is_generated_on_demand() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", migration = true
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus { Active, Suspended }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("pub fn migration_column_def ()"));
        assert!(output.contains(":: sea_orm_migration :: sea_query :: ColumnDef :: new"));
        assert!(
            output.contains(":: sea_orm_migration :: sea_query :: Alias :: new (\"user_status\")")
        );
        assert!(output.contains(". string ()"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn migration_helper_reports_unmapped_db_types() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "HyperCube", migration = true
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus { Active }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("Failed to generate the migration helper"));
        assert!(output.contains("db_type `HyperCube` has no `ColumnDef` mapping"));
        assert!(!output.contains("migration_column_def"));
    }
}
