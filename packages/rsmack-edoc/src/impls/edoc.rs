use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::{fs::package_src_folder, megamac::ExecEnv};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use syn::*;

#[derive(Debug, FromMeta)]
pub struct EdocFieldConcat {
    expr: syn::Expr,
}

#[derive(Debug, FromField, FromVariant)]
#[darling(attributes(edoc))]
pub struct EdocField {
    /// Concatenate members of [syn::ExprTuple] which maybe either [syn::LitStr] or a const [syn::Ident].
    /// Return a `#[doc]` attribute
    #[darling(flatten)]
    concat: EdocFieldConcat,
}
#[derive(Debug, FromMeta)]
pub struct Args {
    from: syn::Path,
}
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(edoc), forward_attrs(allow, doc, cfg))]
pub struct DeriveInputArgs {
    data: ast::Data<EdocField, EdocField>,
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
fn rm_item_fields_attrs<
    R: Fn(
        &mut Field,
        &String,
        // Removed count
        usize,
    ) -> Attribute,
>(
    attrs_to_remove: Vec<String>,
    item: &mut ItemStruct,
    replace_with: R,
) -> ItemStruct {
    for attr_to_rm in &attrs_to_remove {
        let mut removed_count: usize = 0;
        for field in item.fields.iter_mut() {
            let attr_to_rm_idx = field.attrs.iter().position(|a| match a {
                Attribute {
                    meta: Meta::List(MetaList { path, .. }),
                    ..
                } => {
                    let id = path.segments.first().unwrap().ident.clone();
                    &id.to_string() == attr_to_rm
                }
                _ => false,
            });
            if let Some(i) = attr_to_rm_idx {
                let replacement_attr = replace_with(field, attr_to_rm, removed_count);
                field.attrs[i] = replacement_attr;
                removed_count += 1;
            };
        }
    }
    item.clone()
}
/// Execute edoc macro
pub fn exec(args: Args, item: ItemStruct, env: ExecEnv) -> TokenStream {
    let derive_input = syn::parse2::<syn::DeriveInput>(item.to_token_stream()).unwrap();
    let call_site_file_path = call_site_file_path_from_syn_path(args.from);
    let derive_args = match DeriveInputArgs::from_derive_input(&derive_input) {
        Ok(args) => args,
        Err(e) => env
            .logr
            .abort_call_site(format!("Failed to parse macro args: {e}")),
    };

    let derive_args_data = derive_args.data.take_struct();
    let mut evaluated_edoc_fields: Vec<String> = vec![];
    if let Some(fields) = derive_args_data {
        let mut resolved_consts: HashMap<String, String> = HashMap::new();
        resolve_consts(call_site_file_path, &mut resolved_consts, &env);
        for edoc_field in fields {
            let mut evaluated_elems: Vec<String> = vec![];
            match edoc_field.concat.expr {
                Expr::Tuple(tup) => {
                    // Evaluate each element
                    for elem in tup.elems {
                        match elem {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(lit @ LitStr { .. }),
                            ..
                        }) => {
                            evaluated_elems.push(lit.value());
                        }
                        Expr::Path(ExprPath {
                            path: Path { segments, .. },
                            ..
                        }) => {
                            let const_ident = segments.first().unwrap().ident.clone();
                            let const_value = resolved_consts.get(&const_ident.to_string());
                            match const_value {
                                None => env.logr.emit_error(const_ident.span(),format!("Unresolved const ident {const_ident:?}")),
                                Some(value) => evaluated_elems.push(value.clone())
                            }

                        }
                        _ => env.logr.abort_call_site(
                            "Unsupported tuple element, only string literal or ident of a const string",
                        ),
                    }
                    }
                }
                _ => env.logr.abort_call_site(
                    "Only Tuple supported, maybe you are missing a second element".to_string(),
                ),
            }
            let sep = "";
            let evaluated = evaluated_elems.join(sep);
            evaluated_edoc_fields.push(evaluated)
            // env.logr.abort_call_site(&evaluated);
        }
    }

    let field_attr_replacer = |_field: &mut Field, _attr_name: &String, attr_to_rm_idx: usize| {
        let doc_str = &evaluated_edoc_fields[attr_to_rm_idx];
        syn::parse_quote! {#[doc = #doc_str]}
    };
    let edoc_replaced_item =
        rm_item_fields_attrs(vec!["edoc".into()], &mut item.clone(), field_attr_replacer);
    quote! {
        #edoc_replaced_item
    }
}

fn call_site_file_path_from_syn_path(path: syn::Path) -> std::path::PathBuf {
    package_src_folder().join(format!(
        "{}.rs",
        path.to_token_stream().to_string().replace("::", "/")
    ))
}

fn resolve_consts(
    call_site_file_path: std::path::PathBuf,
    resolved_consts: &mut HashMap<String, String>,
    env: &ExecEnv,
) {
    // TODO should be memoized
    if let Ok(lines) = read_lines(call_site_file_path) {
        for line_read in lines {
            match line_read {
                Ok(line) => {
                    const CONST_KW: &str = "const";
                    let trimmed_line = line.trim();
                    if trimmed_line.starts_with(CONST_KW) {
                        let parsed_const_item: syn::ItemConst =
                            syn::parse_str(&line.clone()).unwrap();
                        let const_name = parsed_const_item.ident.to_string();
                        match *parsed_const_item.expr {
                            Expr::Lit(ExprLit { lit:Lit::Str(lit @ LitStr {..}),.. }) => {
                                let _ = resolved_consts.insert(const_name,lit.value());
                            },
                            Expr::Lit(ExprLit { lit:Lit::Bool(lit @ LitBool {..}),.. }) => {
                                let _ = resolved_consts.insert(const_name,lit.value().to_string());
                            },
                            Expr::Lit(ExprLit { lit:Lit::ByteStr(lit @ LitByteStr {..}),.. }) => {
                                let _ = resolved_consts.insert(const_name,format!("{:?}",lit.value()));
                            },
                            x => env.logr.abort_call_site(format!(
                                "Unexpected const item expression here, expected literal or `concat!` (NOT YET SUPPORTED) invocation, received: `{x:?}` at `{line}`"
                            )),
                        }
                    }
                }
                Err(e) => env
                    .logr
                    .abort_call_site(format!("Failed to read line: {e:?}")),
            }
        }
    }
}
