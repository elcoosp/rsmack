//! Procedural macro for generating other procedural macros with documentation and error handling.
//!
//! This module provides the `megamac` procedural macro that can generate different types of
//! procedural macros (function-like, attribute, derive) with automatic documentation generation
//! and proper error handling.

use bon::Builder;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use strum::Display;
use syn::*;

/// Represents the kind of procedural macro to generate.
#[derive(Debug, PartialEq, Display, FromMeta)]
enum MacroKind {
    /// Function-like procedural macro (e.g., `my_macro!()`)
    Func,
    /// Attribute procedural macro (e.g., `#[my_macro]`)
    Attr,
    /// Derive procedural macro (e.g., `#[derive(MyMacro)]`)
    Derive,
}

/// Arguments for configuring the megamac macro generation.
#[derive(Debug, FromMeta)]
pub struct Args {
    /// The type of procedural macro to generate as [`Ident`]
    kind: Ident,

    /// Name of the macro to generate as [`Ident`]
    name: Ident,

    /// The receiver type for attribute macros (only used with [`MacroKind::Attr`])
    #[darling(default)]
    receiver: Option<Ident>,
}

impl From<Ident> for MacroKind {
    /// Converts an [`Ident`] to a [`MacroKind`].
    ///
    /// # Panics
    /// Panics if the identifier doesn't match "Attr", "Func", or "Derive".
    fn from(value: Ident) -> Self {
        match value.to_string().as_str() {
            "Attr" => Self::Attr,
            "Func" => Self::Func,
            "Derive" => Self::Derive,
            _ => panic!("Unsupported MacroKind"),
        }
    }
}

/// Executes the megamac macro to generate a procedural macro implementation.
///
/// This function takes configuration arguments and an execution environment,
/// then generates the appropriate procedural macro implementation with
/// automatically generated documentation.
///
/// # Parameters
/// - `args`: Configuration arguments for the macro
/// - `env`: Execution environment containing context and utilities
///
/// # Returns
/// A [`TokenStream`] containing the generated procedural macro implementation.
pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    let name = args.name.clone();
    let receiver = args.receiver.clone();
    let imports = quote! {
        use proc_macro_error2::*;
    };
    let kind = args.kind.to_string();
    let macro_impl_file_ast = get_macro_impl_file_ast(&args, &env);
    let fields_doc = get_args_fields_doc(&macro_impl_file_ast, &args, &env);

    // Format field documentation for inclusion in the generated macro docs
    let formatted_fields_doc = fields_doc
        .iter()
        .map(|fd| {
            let template_without_ty_qualified_path = format!(
                "* `{}` - {}\n  + type: [`{}`]",
                fd.ident,
                fd.doc.clone().unwrap_or("Not documented".into()),
                fd.ty.to_token_stream().to_string().replace(' ', ""),
            );

            quote! { #[doc = #template_without_ty_qualified_path]}
        })
        .collect::<Vec<_>>();

    let name_str = name.to_string();
    let kind_str = kind.clone();

    // Generate the main documentation for the macro
    let doc = quote! {
        #[doc = concat!(#name_str, " procedural macro (", #kind_str, ").")]
        #[doc ="# Parameters"]
        #(#formatted_fields_doc)*
        #[doc ="# Examples"]
    };

    // Generate the appropriate macro implementation based on the kind
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

    quote! {
        #imports
        #doc
        #macro_impl
    }
}

/// Parses and returns the AST of the macro implementation file.
///
/// # Arguments
/// - `args`: The macro configuration arguments
/// - `env`: The execution environment
///
/// # Panics
/// Panics if the file cannot be read or parsed.
fn get_macro_impl_file_ast(args: &Args, env: &ExecEnv) -> File {
    let package_src_folder = package_src_folder();
    let macro_impl_file_path = package_src_folder
        .join(env.implementations_mod_ident.clone())
        .join(format!("{}.rs", args.name));
    let macro_impl_src =
        std::fs::read_to_string(macro_impl_file_path.clone()).unwrap_or_else(|_| {
            panic!(
                "Failed to get macro_impl_src of {} at {macro_impl_file_path:?}",
                args.name
            )
        });

    match syn::parse_file(&macro_impl_src) {
        Ok(x) => x,
        Err(e) => env.logr.abort_call_site(format!(
            "Failed to parse macro_impl_src {}, this may happen for no real reason in your IDE, check that your project still build with cargo: {e:?}",
            args.name.clone()
        ))
    }
}

/// Documentation information for a field in the macro arguments struct.
#[derive(Debug, Builder)]
struct FieldDoc {
    /// The field identifier
    pub ident: Ident,
    /// The field documentation string, if present
    pub doc: Option<String>,
    /// The field type
    pub ty: Type,
}

/// Extracts documentation for all fields in the macro arguments struct.
///
/// # Arguments
/// - `macro_impl_file_ast`: The AST of the macro implementation file
/// - `args`: The macro configuration arguments
/// - `env`: The execution environment
///
/// # Returns
/// A vector of [`FieldDoc`] containing documentation for each field.
///
/// # Aborts
/// Aborts compilation if the arguments struct cannot be found.
fn get_args_fields_doc(macro_impl_file_ast: &File, args: &Args, env: &ExecEnv) -> Vec<FieldDoc> {
    let args_item = macro_impl_file_ast.items.iter().find(|i| match i {
        Item::Struct(ItemStruct { ident, .. }) => *ident == env.exec_args_ident,
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
                            // Extract documentation from #[doc = "..."] attributes
                            Meta::NameValue(MetaNameValue {
                                value:
                                    Expr::Lit(ExprLit {
                                        lit: Lit::Str(lit_str),
                                        ..
                                    }),
                                path: Path { segments, .. },
                                ..
                            }) => {
                                let PathSegment { ident, .. } = segments.first().unwrap();
                                if *ident == "doc" {
                                    FieldDoc::builder()
                                        .ident(f.ident.clone().unwrap())
                                        .doc(lit_str.value())
                                        .ty(f.ty.clone())
                                        .build()
                                } else {
                                    FieldDoc::builder()
                                        .ident(f.ident.clone().unwrap())
                                        .ty(f.ty.clone())
                                        .build()
                                }
                            }
                            _ => unimplemented!(),
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        };
        fields_doc
    } else {
        env.logr.abort_call_site(format!(
            "Failed to find `{}` struct in `{}` module",
            env.exec_args_ident,
            args.name.clone()
        ));
    }
}
