#![allow(clippy::crate_in_macro_def)]
//! Macro utils to call a given **proc-macro** implementation with attrs & item parsing boilerplate handled with [darling]

/// Use [`call_attr_proc_macro`] on `impls` mod with `Args` args ident
#[macro_export]
macro_rules! call_attr_impls_with_args {
    (
            $exec_fn_mod_ident:ident,
            $item_ty:ty,
            $attr_tok_stream:ident,
            $item_tok_stream:ident
        ) => {
        rsmack_utils::exec::call_attr_proc_macro!(
            impls,
            Args,
            $exec_fn_mod_ident,
            $item_ty,
            $attr_tok_stream,
            $item_tok_stream
        )
    };
}
/// Build an [ExecEnv](crate::megamac::ExecEnv)
#[macro_export]
macro_rules! build_env {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident
    ) => {{
        // This can not be put in the builder otherwise the path is not the caller one
        let module_path = std::module_path!();
        rsmack_utils::megamac::ExecEnv::builder(
            module_path,
            stringify!($implementations_mod_ident),
            stringify!($exec_args_ident),
            stringify!($exec_fn_mod_ident),
        )
        .build()
    }};
}
/// Call an attribute proc-macro implementation function named `exec`.
///
/// This macro is **proc-macro only**.
/// Call a proc-macro implementation function named `exec`,
/// located in the given `implementations module ident`,
/// with the given `args type ident` and `type of the item` for [`syn::parse_macro_input`]
#[macro_export]
macro_rules! call_attr_proc_macro {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident,
        $item_ty:ty,
        $attr_tok_stream:ident,
        $item_tok_stream:ident
    ) => {{
        use darling::*;

        let meta_list = match ast::NestedMeta::parse_meta_list($attr_tok_stream.into()) {
            Ok(v) => v,
            Err(e) => {
                return proc_macro::TokenStream::from(Error::from(e).write_errors());
            }
        };
        let parsed_item = syn::parse_macro_input!($item_tok_stream as $item_ty);
        let parsed_args =
            match crate::$implementations_mod_ident::$exec_fn_mod_ident::$exec_args_ident::from_list(
                &meta_list,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return proc_macro::TokenStream::from(e.write_errors());
                }
            };
        let env = rsmack_utils::build_env!($implementations_mod_ident, $exec_args_ident, $exec_fn_mod_ident);

        crate::$implementations_mod_ident::$exec_fn_mod_ident::exec(
            parsed_args,
            parsed_item,
            env
        )
        .into()
    }};
}
/// Use [`call_func_proc_macro`] on `impls` mod with `Args` args ident
#[macro_export]
macro_rules! call_func_impls_with_args {
    (
            $exec_fn_mod_ident:ident,
            $args_tok_stream:ident
        ) => {
        rsmack_utils::exec::call_func_proc_macro!(impls, Args, $exec_fn_mod_ident, $args_tok_stream)
    };
}
/// Use [`call_derive_proc_macro`] on `impls` mod with `Args` args ident
#[macro_export]
macro_rules! call_derive_impls_with_args {
    (
            $exec_fn_mod_ident:ident,
            $item_tok_stream:ident
        ) => {
        rsmack_utils::exec::call_derive_proc_macro!(
            impls,
            Args,
            $exec_fn_mod_ident,
            $item_tok_stream
        )
    };
}
/// Call a function proc-macro implementation function named `exec`.
#[macro_export]
macro_rules! call_func_proc_macro {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident,
        $args_tok_stream:ident
    ) => {{
        use darling::*;

        let meta_list = match ast::NestedMeta::parse_meta_list($args_tok_stream.into()) {
            Ok(v) => v,
            Err(e) => {
                return proc_macro::TokenStream::from(Error::from(e).write_errors());
            }
        };
        let parsed_args =
            match crate::$implementations_mod_ident::$exec_fn_mod_ident::$exec_args_ident::from_list(
                &meta_list,
            ) {
                Ok(v) => v,
                Err(e) => {
                    return proc_macro::TokenStream::from(e.write_errors());
                }
            };
        let env = rsmack_utils::build_env!($implementations_mod_ident, $exec_args_ident, $exec_fn_mod_ident);

        crate::$implementations_mod_ident::$exec_fn_mod_ident::exec(
            parsed_args,
            env
        )
        .into()
    }};
}

/// Call a derive proc-macro implementation function named `exec`.
#[macro_export]
macro_rules! call_derive_proc_macro {
    (
        $implementations_mod_ident:ident,
        $exec_args_ident:ident,
        $exec_fn_mod_ident:ident,
        $item_tok_stream:ident
    ) => {{
        use darling::*;
        let parsed_item = syn::parse_macro_input!($item_tok_stream as syn::DeriveInput);
        let env = rsmack_utils::build_env!(
            $implementations_mod_ident,
            $exec_args_ident,
            $exec_fn_mod_ident
        );
        crate::$implementations_mod_ident::$exec_fn_mod_ident::exec(parsed_item, env).into()
    }};
}

pub use call_attr_impls_with_args;
pub use call_attr_proc_macro;
pub use call_derive_impls_with_args;
pub use call_derive_proc_macro;
pub use call_func_impls_with_args;
pub use call_func_proc_macro;
