use proc_macro::TokenStream;

mod impls;

use proc_macro2::TokenStream as TokenStream2;
use rsmack_utils::exec::func_entrypoint;
use rsmack_utils::fs::package_src_folder;
use rsmack_utils::megamac::ExecEnv;

/// Executes the megamac macro to generate a procedural macro implementation.
fn exec(args: &impls::megamac::Args, env: &ExecEnv) -> TokenStream2 {
    env.finish_result(impls::megamac::run(args, env, &package_src_folder()))
}

/// Declare a mega macro of any kind with automated parameters documentation.
///
/// `megamac` reads the `impls/<name>.rs` file of the calling crate, extracts the
/// documented `Args` struct of the implementation and generates the proc-macro entrypoint
/// prefixed with a `# Parameters` section built from the `Args` documentation.
///
/// # Parameters
/// * `kind` - The kind of macro to generate, one of `Func`, `Attr` or `Derive`
/// * `name` - The name of the generated macro
/// * `impls` - Ident of the implementations module (defaults to `impls`)
/// * `args` - Ident of the args struct (defaults to `Args`, darling payload mode only)
/// * `doc` - Documentation source of the generated macro: `readme` embeds the calling
///   crate's `../README.md`, `inline` (default) only emits the generated parameters docs,
///   `none` emits no documentation
///
/// The payload protocol is **signature-driven**: when `exec` takes a
/// `proc_macro2::TokenStream` first parameter the payload tokens are forwarded raw
/// (DSL-style macros); otherwise they are darling-parsed into the documented args struct.
/// The parsed item type of `Attr` and `Derive` macros is inferred from the `exec`
/// signature of the implementation, so no receiver configuration is needed. Derive
/// macros receive no payload and get no `# Parameters` section.
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
/// pub fn exec(args: Args, env: &ExecEnv) -> TokenStream {
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
/// megamac!(kind = Attr, name = make_public, doc = readme);
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
    func_entrypoint(
        args.into(),
        concat!(std::module_path!(), "::megamac"),
        |args, env| exec(&args, env),
    )
    .into()
}
