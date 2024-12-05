#![allow(unused)]
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::*;
use rsmack_utils::megamac::ExecEnv;
use syn::*;
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Concatenate members of [syn::ExprTuple] which maybe either [syn::LitStr] or a const [syn::Ident].
    /// Return a `#[doc]` attribute
    concat: syn::Expr,
}
/// Execute edoc macro
pub fn exec(item: DeriveInput, env: ExecEnv) -> TokenStream {
    let mut evaluated_elems: Vec<String> = vec![];
    // Guard against anything not tuple
    // match args.concat {
    //     Expr::Tuple(tup) => {
    //         // Evaluate each element
    //         for elem in tup.elems {
    //             match elem {
    //                 Expr::Lit(ExprLit {
    //                     lit: Lit::Str(lit @ LitStr { .. }),
    //                     ..
    //                 }) => {
    //                     evaluated_elems.push(lit.value());
    //                 }
    //                 Expr::Path(ExprPath {
    //                     path: Path { segments, .. },
    //                     ..
    //                 }) => {
    //                     let msg = format!(
    //                         "Unsupported but known tuple element, received {:#?}",
    //                         segments
    //                     );
    //                     env.logr.abort_call_site(&msg);
    //                 }
    //                 _ => env.logr.abort_call_site(
    //                     "Unsupported tuple element, only string literal or const string ident",
    //                 ),
    //             }
    //         }
    //     }
    //     e => env.logr.abort_call_site(&format!(
    //         "Only Tuple supported, maybe you are missing a second element"
    //     )),
    // }
    let sep = "";
    let evaluated = evaluated_elems.join(sep);
    quote! {
        // #[doc = #evaluated]
    }
}
