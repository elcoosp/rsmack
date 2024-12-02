use ast::NestedMeta;
use darling::*;
use proc_macro::TokenStream;
use proc_macro_error::*;
mod impls;
macro_rules! call_exec_on_impl_with_darling_parsed_args {
    (
        $implementations_mod_ident:ident,
        $attr_proc_macro_mod_ident:ident,
        $item_ty:ty,
        $attr_tok_stream:ident,
        $item_tok_stream:ident
    ) => {{
        let attr_args = match NestedMeta::parse_meta_list($attr_tok_stream.into()) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(Error::from(e).write_errors());
            }
        };
        let parsed_item = syn::parse_macro_input!($item_tok_stream as $item_ty);
        let parsed_args =
            match crate::$implementations_mod_ident::$attr_proc_macro_mod_ident::Args::from_list(
                &attr_args,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return TokenStream::from(e.write_errors());
                }
            };
        crate::$implementations_mod_ident::$attr_proc_macro_mod_ident::exec(
            parsed_args,
            parsed_item,
        )
        .into()
    }};
}

#[proc_macro_error]
#[proc_macro_attribute]
/// Wrap given struct fields into Option<...>
pub fn wrap(attr: TokenStream, item: TokenStream) -> TokenStream {
    call_exec_on_impl_with_darling_parsed_args!(impls, wrap, syn::ItemStruct, attr, item)
}
