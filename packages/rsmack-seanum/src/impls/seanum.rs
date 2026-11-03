use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;

/// Integer Rust types supported as `rs_type` with automatic `TryFrom` impls.
const INTEGER_RS_TYPES: [&str; 10] = [
    "i8", "i16", "i32", "i64", "isize", "u8", "u16", "u32", "u64", "usize",
];

/// The **minimal** default derive list: everything else (`Serialize`, `Deserialize`,
/// `Dummy`, ...) is opt-in via `derives(...)`, so consumers never carry hidden
/// dependencies on `serde` or `fake`.
const DEFAULT_DERIVES: [&str; 7] = [
    "Clone",
    "Debug",
    "PartialEq",
    "Eq",
    "Hash",
    "::sea_orm::EnumIter",
    "::sea_orm::DeriveActiveEnum",
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
    /// Naming case applied to every non-renamed string value
    /// (`rename_all = "camelCase"`)
    #[darling(default)]
    rename_all: Option<syn::LitStr>,
    /// Explicit `enum_name` override (defaults to the snake-cased enum name)
    #[darling(default)]
    enum_name: Option<syn::LitStr>,
}

/// Per-variant configuration parsed from `#[seanum(...)]` attributes.
#[derive(Debug, Default, FromMeta)]
struct VariantArgs {
    /// Database value override (`#[seanum(rename = "active")]`)
    #[darling(default)]
    rename: Option<syn::LitStr>,
}

/// Apply a supported naming case to a value.
///
/// # Errors
/// Unknown case names are reported with the full supported list.
fn apply_case(value: &str, case: &str) -> Result<String, String> {
    match case {
        "snake_case" => Ok(stringcase::snake_case(value)),
        "camelCase" => Ok(stringcase::camel_case(value)),
        "PascalCase" => Ok(stringcase::pascal_case(value)),
        "kebab-case" => Ok(stringcase::kebab_case(value)),
        "SCREAMING_SNAKE_CASE" => Ok(stringcase::macro_case(value)),
        "lowercase" => Ok(value.to_lowercase()),
        "UPPERCASE" => Ok(value.to_uppercase()),
        other => Err(format!(
            "Unsupported `rename_all` value `{other}`, expected `snake_case`, `camelCase`, `PascalCase`, `kebab-case`, `SCREAMING_SNAKE_CASE`, `lowercase` or `UPPERCASE`"
        )),
    }
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
    // The table name: explicit override or the snake-cased enum name.
    let db_enum_name = match &args.enum_name {
        Some(overridden) => overridden.value(),
        None => stringcase::snake_case(&enum_name.to_string()),
    };
    // An unknown case name is fatal: every value would be wrong.
    let rename_all = match &args.rename_all {
        Some(case) => {
            let value = case.value();
            apply_case("_", &value)
                .map_err(|reason| env.logr.error(case.span(), reason))
                .map(|_| Some(value))?
        }
        None => None,
    };
    let integer_rs_type = INTEGER_RS_TYPES.contains(&args.rs_type.to_string().as_str());

    // Collect existing attributes (excluding the seanum attribute)
    let existing_attrs: Vec<_> = attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("seanum"))
        .collect();

    let (values, discriminants_broken) = assign_variant_values(&variants, env)?;
    let variants_with_attrs = build_variant_tokens(
        &variants,
        &values,
        rename_all.as_deref(),
        integer_rs_type,
        env,
    )?;

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

    // Integer `rs_type` gets a `#[repr]` plus a value-based `TryFrom` impl.
    let repr_attr = integer_rs_type.then(|| quote! { #[repr(#rs_type_ident)] });
    let try_from_impl = if integer_rs_type && !discriminants_broken {
        generate_try_from(&enum_name, &variants, &values, &args.rs_type, env)
    } else {
        quote! {}
    };

    let migration_impl = if args.migration {
        generate_migration_column_def(&enum_name, &db_enum_name, &db_type.value(), env)
    } else {
        quote! {}
    };

    let default_derives = DEFAULT_DERIVES
        .iter()
        .map(|derive| syn::parse_str::<syn::Path>(derive).expect("static derive paths are valid"));
    Ok(quote! {
        #(#existing_attrs)*
        #[derive(
            #(#default_derives,)* #(#extra_derives),*
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

/// Assign the integer value of every variant: explicit discriminants win, the implicit
/// sequence continues from the previous value (rustc semantics).
///
/// Returns the values in variant order plus whether the table is broken (unparsable or
/// duplicate values) — a broken table skips the `TryFrom` generation.
fn assign_variant_values(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    env: &ExecEnv,
) -> syn::Result<(Vec<i128>, bool)> {
    let mut next_implicit: i128 = 0;
    let mut seen_values: std::collections::HashSet<i128> = std::collections::HashSet::new();
    let mut values: Vec<i128> = Vec::with_capacity(variants.len());
    let mut discriminants_broken = false;
    for variant in variants {
        let mut value = next_implicit;
        if let Some((_, expr)) = &variant.discriminant {
            if let Some(parsed) = parse_discriminant(expr) {
                value = parsed;
            } else {
                env.logr.emit_error(
                    expr.span(),
                    "Failed to parse the variant discriminant: only integer literals (with an optional leading `-`) are supported",
                );
                discriminants_broken = true;
            }
        }
        if !seen_values.insert(value) {
            env.logr.emit_error(
                variant.span(),
                format!(
                    "Duplicate variant value `{value}`: discriminants must be unique for the generated `TryFrom`"
                ),
            );
            discriminants_broken = true;
        }
        next_implicit = value
            .checked_add(1)
            .ok_or_else(|| env.logr.error(variant.span(), "Variant value overflow"))?;
        values.push(value);
    }
    Ok((values, discriminants_broken))
}

/// Generate the tokens of every variant: existing attributes are preserved, the
/// `#[seanum(...)]` configuration is consumed, and the database value attribute is
/// generated (`num_value` for integer `rs_type`, `string_value` otherwise).
fn build_variant_tokens(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    values: &[i128],
    rename_all: Option<&str>,
    integer_rs_type: bool,
    env: &ExecEnv,
) -> syn::Result<Vec<TokenStream>> {
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
        let value_lit = if let Some(renamed) = &rename {
            renamed.clone()
        } else {
            let name = variant_name.to_string();
            let cased = rename_all
                .map(|case| apply_case(&name, case))
                .transpose()
                .map_err(|reason| env.logr.call_site_error(reason))?;
            syn::LitStr::new(
                cased.as_deref().unwrap_or(name.as_str()),
                variant_name.span(),
            )
        };
        let value_lit_int = syn::LitInt::new(&values[index].to_string(), variant_name.span());
        let sea_orm_attr = if integer_rs_type {
            quote! { #[sea_orm(num_value = #value_lit_int)] }
        } else {
            quote! { #[sea_orm(string_value = #value_lit)] }
        };

        variants_with_attrs.push(quote! {
            #(#existing_variant_attrs)*
            #sea_orm_attr
            #variant_name
        });
    }
    Ok(variants_with_attrs)
}

/// Parse an explicit variant discriminant: an integer literal, optionally negated.
fn parse_discriminant(expr: &syn::Expr) -> Option<i128> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        }) => lit.base10_parse::<i128>().ok(),
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr: inner,
            ..
        }) => match inner.as_ref() {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit),
                ..
            }) => lit.base10_parse::<i128>().ok().map(|value| -value),
            _ => None,
        },
        _ => None,
    }
}

/// Generate a value-based `TryFrom<rs_type>` impl for an integer-backed enum.
///
/// Requires unit variants and unique values; when violated, a non-fatal error is
/// emitted and no impl is generated.
fn generate_try_from(
    enum_name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    values: &[i128],
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
    }
    let arms = variants.iter().zip(values).map(|(variant, value)| {
        let ident = &variant.ident;
        let value_lit = syn::LitInt::new(&value.to_string(), variant.ident.span());
        quote! { #value_lit => ::core::result::Result::Ok(Self::#ident), }
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
        ExecEnv::new("rsmack_seanum::impls")
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
        // v2: the minimal default derive list — no hidden serde/fake derives
        assert!(output.contains("Clone , Debug , PartialEq , Eq , Hash , :: sea_orm :: EnumIter , :: sea_orm :: DeriveActiveEnum"));
        assert!(!output.contains("Serialize"));
        assert!(!output.contains("Dummy"));
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
        assert!(output.contains("DeriveActiveEnum , Display , Eq)"));
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
    fn rename_all_applies_to_non_renamed_values() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", rename_all = "kebab-case"
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus {
                #[seanum(rename = "ACTIVE")]
                Active,
                SendEmail,
            }
        };
        let output = exec(args, item, &env()).to_string();
        // the explicit rename wins over rename_all
        assert!(output.contains("string_value = \"ACTIVE\""));
        assert!(output.contains("string_value = \"send-email\""));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn rename_all_supports_every_case() {
        for (case, expected) in [
            ("snake_case", "send_email"),
            ("camelCase", "sendEmail"),
            ("PascalCase", "SendEmail"),
            ("kebab-case", "send-email"),
            ("SCREAMING_SNAKE_CASE", "SEND_EMAIL"),
            ("lowercase", "sendemail"),
            ("UPPERCASE", "SENDEMAIL"),
        ] {
            let args = args_from(quote! {
                rs_type = "String", db_type = "String", rename_all = #case
            });
            let item: syn::ItemEnum = syn::parse_quote! {
                enum Actions { SendEmail }
            };
            let output = exec(args, item, &env()).to_string();
            assert!(
                output.contains(&format!("string_value = \"{expected}\"")),
                "case {case} failed, got: {output}"
            );
            assert!(!output.contains("compile_error"));
        }
    }

    #[test]
    fn unknown_rename_all_case_is_fatal() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", rename_all = "nopeCase"
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Actions { SendEmail }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Unsupported `rename_all` value `nopeCase`"));
    }

    #[test]
    fn enum_name_override_is_honored() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", enum_name = "user_state"
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus { Active }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("enum_name = \"user_state\""));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn integer_rs_type_generates_repr_num_values_and_try_from() {
        let args = args_from(quote! { rs_type = "i32", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Priority { Low, Medium, High }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("# [repr (i32)]"));
        assert!(output.contains("# [sea_orm (num_value = 0)] Low"));
        assert!(output.contains("# [sea_orm (num_value = 1)] Medium"));
        assert!(output.contains("# [sea_orm (num_value = 2)] High"));
        assert!(output.contains("impl :: core :: convert :: TryFrom < i32 > for Priority"));
        assert!(output.contains("0 => :: core :: result :: Result :: Ok (Self :: Low)"));
        assert!(output.contains("Invalid integer value for {}"));
        assert!(!output.contains("compile_error"));
    }

    /// Explicit discriminants are honored by `num_value` and `TryFrom`.
    #[test]
    fn explicit_discriminants_are_honored() {
        let args = args_from(quote! { rs_type = "u8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Level { Low = 10, Medium, High = 100 }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("# [sea_orm (num_value = 10)] Low"));
        // the implicit sequence continues from the previous discriminant
        assert!(output.contains("# [sea_orm (num_value = 11)] Medium"));
        assert!(output.contains("# [sea_orm (num_value = 100)] High"));
        assert!(output.contains("10 => :: core :: result :: Result :: Ok (Self :: Low)"));
        assert!(output.contains("11 => :: core :: result :: Result :: Ok (Self :: Medium)"));
        assert!(output.contains("100 => :: core :: result :: Result :: Ok (Self :: High)"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn negative_discriminants_are_supported() {
        let args = args_from(quote! { rs_type = "i8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Signed { Minus = -1, Zero }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(
            output.contains("# [sea_orm (num_value = - 1)] Minus"),
            "got: {output}"
        );
        assert!(output.contains("# [sea_orm (num_value = 0)] Zero"));
        assert!(!output.contains("compile_error"));
    }

    #[test]
    fn duplicate_discriminants_are_collected_errors() {
        let args = args_from(quote! { rs_type = "u8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Clash { A = 1, B, C = 2 }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Duplicate variant value `2`"));
        // the TryFrom impl is skipped when values collide
        assert!(!output.contains("convert :: TryFrom"));
    }

    #[test]
    fn non_literal_discriminants_are_collected_errors() {
        let args = args_from(quote! { rs_type = "u8", db_type = "Integer" });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum Weird { A = OTHER_CONST }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to parse the variant discriminant"));
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

    /// The migration alias follows the `enum_name` override.
    #[test]
    fn migration_helper_follows_enum_name_override() {
        let args = args_from(quote! {
            rs_type = "String", db_type = "String", migration = true, enum_name = "custom_name"
        });
        let item: syn::ItemEnum = syn::parse_quote! {
            enum UserStatus { Active }
        };
        let output = exec(args, item, &env()).to_string();
        assert!(
            output.contains(":: sea_orm_migration :: sea_query :: Alias :: new (\"custom_name\")")
        );
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

    #[test]
    fn discriminant_parsing_covers_literals_and_negation() {
        let lit: syn::Expr = syn::parse_quote! { 7 };
        assert_eq!(parse_discriminant(&lit), Some(7));
        let neg: syn::Expr = syn::parse_quote! { -7 };
        assert_eq!(parse_discriminant(&neg), Some(-7));
        let bad: syn::Expr = syn::parse_quote! { SOME_CONST };
        assert_eq!(parse_discriminant(&bad), None);
        let float: syn::Expr = syn::parse_quote! { 1.5 };
        assert_eq!(parse_discriminant(&float), None);
        let bad_neg: syn::Expr = syn::parse_quote! { -SOME_CONST };
        assert_eq!(parse_discriminant(&bad_neg), None);
    }

    #[test]
    fn apply_case_maps_the_supported_set() {
        assert_eq!(
            apply_case("SendEmail", "snake_case").expect("case"),
            "send_email"
        );
        assert_eq!(
            apply_case("send_email", "camelCase").expect("case"),
            "sendEmail"
        );
        assert_eq!(
            apply_case("send_email", "PascalCase").expect("case"),
            "SendEmail"
        );
        assert_eq!(
            apply_case("send_email", "SCREAMING_SNAKE_CASE").expect("case"),
            "SEND_EMAIL"
        );
        assert_eq!(
            apply_case("SendEmail", "lowercase").expect("case"),
            "sendemail"
        );
        assert_eq!(
            apply_case("SendEmail", "UPPERCASE").expect("case"),
            "SENDEMAIL"
        );
        let err = apply_case("x", "nopeCase").expect_err("unknown case");
        assert!(err.contains("Unsupported `rename_all` value `nopeCase`"));
    }
}
