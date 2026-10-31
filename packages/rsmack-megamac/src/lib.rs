use proc_macro::TokenStream;

mod impls;

use rsmack_utils::exec::call_func_impls_with_args;

/// Declare a mega macro of any kind with automated parameters documentation.
///
/// `megamac` reads the `impls/<name>.rs` file of the calling crate, extracts the
/// documented `Args` struct of the implementation and generates the proc-macro entrypoint
/// prefixed with a `# Parameters` section built from the `Args` documentation.
///
/// # Parameters
/// * `kind` - The kind of macro to generate, one of `Func`, `Attr` or `Derive`
/// * `name` - The name of the generated macro
/// * `receiver` - The `syn` item type received by `Attr` macros (defaults to `ItemStruct`)
///
/// # Examples
///
/// Generate a function-like macro generating a greeting constant (full implementation in
/// `packages/rsmack-example-macros/src/impls/greeting.rs`):
///
/// ```rust,ignore
/// // src/lib.rs of your proc-macro crate
/// use rsmack_megamac::megamac;
///
/// mod impls;
///
/// megamac!(kind = Func, name = greeting);
///
/// // src/impls/greeting.rs
/// use darling::FromMeta;
/// use proc_macro2::TokenStream;
/// use rsmack_utils::megamac::ExecEnv;
///
/// /// Greeting configuration
/// #[derive(Debug, FromMeta)]
/// pub struct Args {
///     /// Recipient of the greeting
///     pub who: String,
/// }
///
/// pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
///     let who = args.who;
///     env.finish(quote::quote! {
///         pub const GREETING: &str = concat!("Hello ", #who, "!");
///     })
/// }
///
/// // Somewhere in a consumer crate:
/// // greeting!(who = "world");
/// // assert_eq!(GREETING, "Hello world!");
/// ```
///
/// Generate an attribute macro making every field public (see
/// `packages/rsmack-example-macros/src/impls/make_public.rs`):
///
/// ```rust,ignore
/// megamac!(kind = Attr, name = make_public, receiver = ItemStruct);
///
/// // #[make_public]
/// // struct Demo { a: u8 }
/// // let demo = Demo { a: 7 };
/// ```
///
/// Generate a derive macro (the generated derive is the `PascalCase` of `name`, see
/// `packages/rsmack-example-macros/src/impls/type_name.rs`):
///
/// ```rust,ignore
/// megamac!(kind = Derive, name = type_name);
///
/// // #[derive(TypeName)]
/// // struct User;
/// // assert_eq!(User::type_name(), "User");
/// ```
#[proc_macro]
pub fn megamac(args: TokenStream) -> TokenStream {
    call_func_impls_with_args!(megamac, args)
}
