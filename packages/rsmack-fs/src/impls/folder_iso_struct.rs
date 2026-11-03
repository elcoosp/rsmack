/// A procedural macro for generating structs that mirror folder structures at compile
/// time.
///
/// The macro scans the folder **at expansion time** and emits the mirrored struct
/// inline: one attribute, zero build script, no `OUT_DIR`, no `include!`. Files become
/// struct fields (each module file must expose a `PascalCase`-named type); with
/// `recursive = true`, subdirectories become nested modules holding their own mirrored
/// struct. The declared fields of the annotated struct are a sketch only — the mirror
/// is always generated from the folder itself.
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use rsmack_utils::fs::{package_src_folder, parse_id_maybe_raw, scan_modules};
use rsmack_utils::megamac::ExecEnv;
use syn::spanned::Spanned;
use syn::{ItemStruct, Visibility};

/// Arguments for the `folder_iso_struct` macro
#[derive(Debug, FromMeta)]
pub struct Args {
    /// Name of the folder to mirror, relative to the calling crate's `src/`
    /// (e.g. `"levels"` or `"ui/screens"`).
    folder: String,

    /// Visibility of every generated item (e.g. `"pub(crate)"`). Defaults to the
    /// visibility declared on the annotated struct.
    #[darling(default)]
    vis: Option<String>,

    /// Mirror subdirectories as nested modules with their own struct
    /// (`recursive = true`).
    #[darling(default)]
    recursive: bool,

    /// Extra derives injected on every generated struct (`derives("Debug", "Clone")`).
    #[darling(default)]
    derives: Vec<syn::LitStr>,
}

/// Executes the `folder_iso_struct` macro transformation
///
/// # Returns
/// Returns a `TokenStream` containing the mirrored struct (and, in recursive mode, its
/// nested modules) emitted inline, followed by nothing else — no build step involved.
pub fn exec(args: Args, item: ItemStruct, env: &ExecEnv) -> TokenStream {
    env.finish_result(run(args, item, env))
}

fn run(args: Args, item: ItemStruct, env: &ExecEnv) -> syn::Result<TokenStream> {
    let Args {
        folder,
        vis,
        recursive,
        derives,
    } = args;
    // Ensure we're only working with structs that have named fields
    if !matches!(item.fields, syn::Fields::Named(_)) {
        return Err(env
            .logr
            .error(item.fields.span(), "Only named struct supported"));
    }

    // The annotated struct's own visibility is the default for every generated item;
    // `vis` overrides it explicitly.
    let visibility: Visibility = match &vis {
        Some(vis_str) => syn::parse_str(vis_str).map_err(|_| {
            env.logr.call_site_error(format!(
                "Invalid `vis` value `{vis_str}`, expected e.g. `pub` or `pub(crate)`"
            ))
        })?,
        None => item.vis.clone(),
    };

    let extra_derives: Vec<syn::Path> = derives
        .iter()
        .map(|derive| syn::parse_str::<syn::Path>(&derive.value()))
        .collect::<syn::Result<_>>()
        .map_err(|e| {
            env.logr
                .call_site_error(format!("Invalid `derives` entry: {e}"))
        })?;

    let syn::ItemStruct { attrs, ident, .. } = item;
    let derives_attr = quote! { #(#attrs)* #[derive(#(#extra_derives),*)] };

    // Scan the folder of the calling crate (`$CARGO_MANIFEST_DIR/src/<folder>`) and
    // emit the mirror inline.
    let folder_path = package_src_folder().join(&folder);
    let tree = scan_modules(&folder_path, recursive).map_err(|reason| {
        env.logr.call_site_error(format!(
            "Failed to scan the folder to mirror `{}`: {reason}",
            folder_path.display()
        ))
    })?;

    // The mirror references `crate::<folder>::<mod>::<Type>`: the folder segments map
    // to the module path (`"ui/screens"` -> `crate::ui::screens`).
    let folder_segs: Vec<syn::Ident> = folder
        .split('/')
        .filter(|seg| !seg.is_empty())
        .map(parse_id_maybe_raw)
        .collect();
    let crate_path = quote! { crate::#(#folder_segs)::* };
    let rendered = tree.render(&ident, &crate_path, &derives_attr, &visibility);
    Ok(quote! { #rendered })
}

#[cfg(test)]
mod tests {
    use super::*;
    use darling::ast::NestedMeta;

    fn args_from(tokens: TokenStream) -> Args {
        let meta_list = NestedMeta::parse_meta_list(tokens).expect("failed to parse meta list");
        Args::from_list(&meta_list).expect("failed to parse args")
    }

    fn env() -> ExecEnv {
        ExecEnv::new("rsmack_fs::impls")
    }

    /// Serialize the tests mutating process-wide environment variables.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Create a folder tree to mirror under `root/src/<folder>`.
    fn setup_folder(root: &std::path::Path, folder: &str, recursive_files: bool) {
        let widgets = root.join("src").join(folder);
        std::fs::create_dir_all(&widgets).expect("create folder");
        std::fs::write(widgets.join("button.rs"), "").expect("write button");
        std::fs::write(widgets.join("mod.rs"), "").expect("write mod");
        if recursive_files {
            let panels = widgets.join("panels");
            std::fs::create_dir_all(&panels).expect("create panels");
            std::fs::write(panels.join("sidebar.rs"), "").expect("write sidebar");
        }
    }

    #[test]
    fn generates_the_mirror_inline() {
        let _guard = env_guard();
        let root = std::env::temp_dir().join("rsmack-fs-inline");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create root");
        setup_folder(&root, "levels", false);

        let args = args_from(quote! { folder = "levels" });
        let item: ItemStruct = syn::parse_quote! {
            #[derive(Debug)]
            struct Levels {
                alpha: Alpha,
            }
        };
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
        let output = exec(args, item, &env()).to_string();
        // The mirror is emitted inline: no generate() method, no OUT_DIR involved.
        assert!(output.contains("# [derive (Debug)]"));
        assert!(output.contains("struct Levels"));
        assert!(output.contains("button : crate :: levels :: button :: Button"));
        assert!(!output.contains("fn generate"));
        assert!(!output.contains("rsmack_utils"));
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn vis_recursive_and_derives_are_applied() {
        let _guard = env_guard();
        let root = std::env::temp_dir().join("rsmack-fs-options");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create root");
        setup_folder(&root, "ui/screens", true);

        let args = args_from(quote! {
            folder = "ui/screens", recursive = true, derives("Clone")
        });
        let item: ItemStruct = syn::parse_quote! {
            pub(crate) struct Screens {}
        };
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
        let output = exec(args, item, &env()).to_string();
        // The declared visibility is used for every generated item; the derives option
        // lands next to the annotated struct's own attributes.
        assert!(output.contains("pub (crate) struct Screens"));
        assert!(output.contains("# [derive (Clone)]"));
        // the `panels` subdirectory becomes a nested module (recursive mode)
        assert!(output.contains("pub (crate) mod panels"));
        assert!(output.contains("pub (crate) struct Panels"));
        assert!(output.contains("button : crate :: ui :: screens :: button :: Button"));
        assert!(
            output.contains("sidebar : crate :: ui :: screens :: panels :: sidebar :: Sidebar")
        );
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn vis_option_overrides_the_declared_visibility() {
        let _guard = env_guard();
        let root = std::env::temp_dir().join("rsmack-fs-vis-override");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create root");
        setup_folder(&root, "levels", false);

        let args = args_from(quote! { folder = "levels", vis = "pub(super)" });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {}
        };
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("pub (super) struct Levels"));
        assert!(output.contains("button : crate :: levels :: button :: Button"));
        assert!(!output.contains("compile_error"));

        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn invalid_vis_is_a_fatal_error() {
        let _guard = env_guard();
        let args = args_from(quote! {
            folder = "levels", vis = "not a visibility"
        });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {}
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Invalid `vis` value"));
    }

    #[test]
    fn invalid_derives_entry_is_a_fatal_error() {
        let _guard = env_guard();
        let args = args_from(quote! { folder = "levels", derives("not a path !!!") });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {}
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Invalid `derives` entry"));
    }

    #[test]
    fn tuple_struct_is_a_fatal_error() {
        let _guard = env_guard();
        let args = args_from(quote! { folder = "levels" });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels(Alpha);
        };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Only named struct supported"));
    }

    #[test]
    fn missing_folder_is_a_fatal_error() {
        let _guard = env_guard();
        let root = std::env::temp_dir().join("rsmack-fs-missing");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create root");
        let args = args_from(quote! { folder = "definitely_missing" });
        let item: ItemStruct = syn::parse_quote! {
            struct Levels {}
        };
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", &root) };
        let output = exec(args, item, &env()).to_string();
        assert!(output.contains("compile_error"));
        assert!(output.contains("Failed to scan the folder to mirror"));
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR")) };
        std::fs::remove_dir_all(&root).ok();
    }
}
