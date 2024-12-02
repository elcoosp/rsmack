pub mod exec {
    #[macro_export]
    /// Call [`crate::exec::call`] on `impls` mod with `Args` args ident
    macro_rules! call_impls_with_args {
        (
            $exec_fn_mod_ident:ident,
            $item_ty:ty,
            $attr_tok_stream:ident,
            $item_tok_stream:ident
        ) => {
            exec::call!(
                impls,
                Args,
                $exec_fn_mod_ident,
                $item_ty,
                $attr_tok_stream,
                $item_tok_stream
            )
        };
    }

    /// Only for proc-macro attributes
    #[macro_export]
    macro_rules! call {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident,
        $item_ty:ty,
        $attr_tok_stream:ident,
        $item_tok_stream:ident
    ) => {{
        use ast::NestedMeta;
        use darling::*;

        let attr_args = match NestedMeta::parse_meta_list($attr_tok_stream.into()) {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from(Error::from(e).write_errors());
            }
        };
        let parsed_item = syn::parse_macro_input!($item_tok_stream as $item_ty);
        let parsed_args =
            match crate::$implementations_mod_ident::$exec_fn_mod_ident::$exec_args_ident::from_list(
                &attr_args,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return TokenStream::from(e.write_errors());
                }
            };
        crate::$implementations_mod_ident::$exec_fn_mod_ident::exec(
            parsed_args,
            parsed_item,
        )
        .into()
    }};
}
    pub use call;
    pub use call_impls_with_args;
}
