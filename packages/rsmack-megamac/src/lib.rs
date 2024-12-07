use proc_macro::TokenStream;
use proc_macro_error2::*;
use rsmack_utils::*;
mod impls;

/// Declare a mega macro of any kind with automated parameters documentation
#[proc_macro_error]
#[proc_macro]
pub fn megamac(args: TokenStream) -> TokenStream {
    exec::call_func_impls_with_args!(megamac, args)
}
