//! Function-like macro: generates a `GREETING` constant.

use darling::FromMeta;
use proc_macro2::TokenStream;
use rsmack_utils::megamac::ExecEnv;

/// Greeting configuration
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Recipient of the greeting
    pub who: String,
}

pub fn exec(args: Args, env: &ExecEnv) -> TokenStream {
    let who = args.who;
    env.finish(quote::quote! {
        pub const GREETING: &str = concat!("Hello ", #who, "!");
    })
}
