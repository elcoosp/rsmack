//! [rsmack-fs](../rsmack_fs/index.html) related utils

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use bon::builder;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Get the directory (workspace root) from which we are compiling.
///
/// Walks up from `OUT_DIR` until a directory containing a `Cargo.toml` is found, which
/// makes it robust to custom target directory layouts (`CARGO_TARGET_DIR`, coverage
/// runs, ...). The result is memoized.
///
/// # Panics
/// If no parent directory of `OUT_DIR` contains a `Cargo.toml`.
#[must_use]
pub fn calling_crate_dir() -> &'static Path {
    static CALLING_CRATE_DIR: OnceLock<PathBuf> = OnceLock::new();
    CALLING_CRATE_DIR
        .get_or_init(|| workspace_root_from(Path::new(env!("OUT_DIR"))))
        .as_path()
}

/// Walk up from `start` until a directory containing a `Cargo.toml` is found.
///
/// # Panics
/// If no parent directory of `start` contains a `Cargo.toml`.
fn workspace_root_from(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir;
        }
        assert!(dir.pop(), "Failed to find a workspace root above OUT_DIR");
    }
}

/// A folder entry mapped to a struct field: the module identifier and the mirrored
/// `PascalCase` type identifier.
struct ModuleField {
    mod_ident: Ident,
    ty_ident: Ident,
}

/// A scanned directory: its own fields (files) and, in recursive mode, one nested
/// [`FolderNode`] per subdirectory.
struct FolderNode {
    fields: Vec<ModuleField>,
    dirs: Vec<(Ident, FolderNode)>,
}

/// List the directory entries of `mods_folder`, sorted by name for deterministic output.
///
/// # Panics
/// - If `mods_folder` cannot be read
fn sorted_entries(mods_folder: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(mods_folder).expect("Failed to read the folder to mirror");
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.expect("Failed to read a folder entry").path())
        .collect();
    paths.sort();
    paths
}

/// Scan a folder of rust modules into a [`FolderNode`].
///
/// `mod.rs` files are skipped. Files always become fields; directories become fields
/// too in flat mode (legacy behavior) or nested [`FolderNode`]s when `recursive` is set.
///
/// # Panics
/// - If a file stem is not valid UTF-8
fn scan_folder(mods_folder: &Path, recursive: bool) -> FolderNode {
    let opts = stringcase::Options {
        separate_before_non_alphabets: true,
        separate_after_non_alphabets: true,
        separators: "_",
        keep: "",
    };
    let mut node = FolderNode {
        fields: Vec::new(),
        dirs: Vec::new(),
    };
    for path in sorted_entries(mods_folder) {
        push_entry(&mut node, &path, recursive, &opts);
    }
    node
}

/// Mirror one directory entry into `node`: skipped for `mod.rs`, a nested directory
/// mirror in recursive mode, a [`ModuleField`] otherwise.
///
/// # Panics
/// - If the file stem is not valid UTF-8
fn push_entry(node: &mut FolderNode, path: &Path, recursive: bool, opts: &stringcase::Options) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("Non UTF-8 or stem-less module file name");
    // `mod.rs` files are module glue, not mirrored types.
    if stem == "mod" {
        return;
    }
    let mod_ident = parse_id_maybe_raw(stem);
    if path.is_dir() && recursive {
        let child = scan_folder(path, recursive);
        node.dirs.push((mod_ident, child));
    } else {
        let ty_ident = parse_id_maybe_raw(&stringcase::pascal_case_with_options(stem, opts));
        node.fields.push(ModuleField {
            mod_ident,
            ty_ident,
        });
    }
}

/// Render the struct mirroring `node`, prefixed by `crate_path` (e.g. `crate::folder`).
///
/// Nested directories are rendered as inline `pub mod` blocks, each containing their own
/// mirrored struct.
fn render_struct(
    node: &FolderNode,
    name: &Ident,
    crate_path: &TokenStream,
    pre: &TokenStream,
    vis: &syn::Visibility,
) -> TokenStream {
    let fields_ts = node.fields.iter().map(|f| {
        let mod_ident = &f.mod_ident;
        let ty_ident = &f.ty_ident;
        let ty_path = quote! { #crate_path::#mod_ident::#ty_ident };
        quote! { #mod_ident: #ty_path }
    });
    let mods_ts = node.dirs.iter().map(|(mod_ident, child)| {
        let child_name = parse_id_maybe_raw(&stringcase::pascal_case(&mod_ident.to_string()));
        let child_path = quote! { #crate_path::#mod_ident };
        let child_struct = render_struct(child, &child_name, &child_path, pre, vis);
        quote! {
            #vis mod #mod_ident {
                #child_struct
            }
        }
    });
    quote! {
        #pre
        #vis struct #name {
            #(#fields_ts,)*
            #(#mods_ts)*
        }
    }
}

/// Generate a struct which fields match a folder of rust modules, each exposing at
/// least a same file named **`PascalCase`** type.
///
/// With `recursive = true`, subdirectories become nested `pub mod` blocks holding their
/// own mirrored struct. `vis` controls the visibility of every generated item
/// (`"pub"`, `"pub(crate)"`, ...).
///
/// # Panics
/// - If `vis` is not a valid [`syn::Visibility`]
/// - If the mirrored folder cannot be read or contains non UTF-8 file names
#[builder]
pub fn folder_iso_struct(
    name: &str,
    pre: &TokenStream,
    folder: &str,
    from_crate: &str,
    #[builder(default = "pub")] vis: &str,
    #[builder(default = false)] recursive: bool,
    #[builder(default = false)] log_enabled: bool,
) {
    use build_print::custom_println;

    let visibility: syn::Visibility = syn::parse_str(vis).unwrap_or_else(|_| {
        panic!("Invalid `vis` value `{vis}`, expected e.g. `pub` or `pub(crate)`")
    });
    let ccd = calling_crate_dir();
    let mods_folder_path = ccd.join(from_crate).join("src").join(folder);
    let root = scan_folder(&mods_folder_path, recursive);
    if log_enabled {
        for f in &root.fields {
            custom_println!(
                "folder_iso_struct",
                green,
                "{} -> crate::{}::{}::{}",
                name,
                folder,
                f.mod_ident,
                f.ty_ident
            );
        }
    }
    let name_id = parse_id_maybe_raw(name);
    let folder_id = parse_id_maybe_raw(folder);
    let crate_path = quote! { crate::#folder_id };
    let output = render_struct(&root, &name_id, &crate_path, pre, &visibility);
    generate_file(format!("{name}.rs"), output.to_string().as_bytes());
}

/// Parse an identifier, falling back to a raw identifier for keywords.
fn parse_id_maybe_raw(s: &str) -> Ident {
    syn::parse_str::<Ident>(s).unwrap_or_else(|_| Ident::new_raw(s, Span::call_site()))
}

/// Generate file in `OUT_DIR`
///
/// # Panics
/// - If `OUT_DIR` is not set
/// - If file creation or write fails
pub fn generate_file<P: AsRef<Path>>(path: P, text: &[u8]) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = PathBuf::from(out_dir).join(path);
    let mut f = File::create(&dest_path).expect("Failed to create the generated file");
    f.write_all(text)
        .expect("Failed to write the generated file");
}

/// Get the package source folder from `CARGO_MANIFEST_DIR` at runtime
///
/// # Panics
/// If `CARGO_MANIFEST_DIR` is not set
#[must_use]
pub fn package_src_folder() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("src")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    fn tempdir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rsmack-utils-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create tempdir");
        dir
    }

    /// Serialize the tests mutating process-wide environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn calling_crate_dir_points_to_existing_workspace_root() {
        let dir = calling_crate_dir();
        assert!(dir.exists(), "calling_crate_dir should exist: {dir:?}");
        assert!(dir.join("Cargo.toml").exists(), "expected workspace root");
    }

    #[test]
    fn workspace_root_from_walks_up_to_manifest() {
        let root = tempdir("workspace-root");
        std::fs::write(root.join("Cargo.toml"), "").expect("write manifest");
        let deep = root.join("target/debug/build/pkg-hash/out");
        std::fs::create_dir_all(&deep).expect("create deep dir");
        assert_eq!(workspace_root_from(&deep), root);
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    #[should_panic(expected = "Failed to find a workspace root above OUT_DIR")]
    fn workspace_root_from_panics_without_manifest() {
        let root = tempdir("no-manifest");
        let deep = root.join("target/debug/build/pkg-hash/out");
        std::fs::create_dir_all(&deep).expect("create deep dir");
        let _ = workspace_root_from(&deep);
    }

    #[test]
    fn scan_folder_flat_mode_maps_dirs_to_fields() {
        let dir = tempdir("scan-flat");
        for file in ["alpha.rs", "beta_gamma.rs", "mod.rs", "snake_case.rs"] {
            std::fs::write(dir.join(file), "").expect("write module file");
        }
        std::fs::create_dir(dir.join("nested_dir")).expect("create nested dir");
        let node = scan_folder(&dir, false);
        let mut rendered: Vec<(String, String)> = node
            .fields
            .iter()
            .map(|f| (f.mod_ident.to_string(), f.ty_ident.to_string()))
            .collect();
        rendered.sort();
        assert_eq!(
            rendered,
            vec![
                ("alpha".into(), "Alpha".into()),
                ("beta_gamma".into(), "BetaGamma".into()),
                ("nested_dir".into(), "NestedDir".into()),
                ("snake_case".into(), "SnakeCase".into()),
            ]
        );
        assert!(node.dirs.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_folder_recursive_mode_nests_directories() {
        let dir = tempdir("scan-recursive");
        std::fs::write(dir.join("alpha.rs"), "").expect("write alpha");
        let nested = dir.join("nested_dir");
        std::fs::create_dir(&nested).expect("create nested dir");
        std::fs::write(nested.join("deep_file.rs"), "").expect("write deep file");
        let node = scan_folder(&dir, true);
        assert_eq!(node.fields.len(), 1);
        assert_eq!(node.dirs.len(), 1);
        let (ident, child) = &node.dirs[0];
        assert_eq!(ident.to_string(), "nested_dir");
        assert_eq!(child.fields[0].ty_ident.to_string(), "DeepFile");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[should_panic(expected = "Failed to read the folder to mirror")]
    fn scan_folder_panics_on_missing_folder() {
        let _ = scan_folder(Path::new("/definitely/not/a/folder"), false);
    }

    #[test]
    #[should_panic(expected = "Non UTF-8 or stem-less module file name")]
    fn scan_folder_panics_on_non_utf8_file_name() {
        use std::os::unix::ffi::OsStrExt;
        let dir = tempdir("non-utf8");
        std::fs::create_dir_all(&dir).expect("create dir");
        let bad_name = std::ffi::OsStr::from_bytes(b"\xff\xfe.rs");
        std::fs::write(dir.join(bad_name), b"").expect("write non utf-8 file");
        let _ = scan_folder(&dir, false);
    }

    #[test]
    fn render_struct_renders_fields_with_pre_attrs() {
        let node = FolderNode {
            fields: vec![ModuleField {
                mod_ident: parse_id_maybe_raw("alpha"),
                ty_ident: parse_id_maybe_raw("Alpha"),
            }],
            dirs: Vec::new(),
        };
        let vis: syn::Visibility = syn::parse_str("pub").expect("vis");
        let crate_path = quote! { crate::levels };
        let rendered = render_struct(
            &node,
            &parse_id_maybe_raw("Levels"),
            &crate_path,
            &quote! { #[derive(Debug)] },
            &vis,
        )
        .to_string();
        assert!(rendered.contains("# [derive (Debug)]"));
        assert!(rendered.contains("pub struct Levels"));
        assert!(rendered.contains("alpha : crate :: levels :: alpha :: Alpha"));
    }

    #[test]
    fn render_struct_renders_nested_modules_recursively() {
        let child = FolderNode {
            fields: vec![ModuleField {
                mod_ident: parse_id_maybe_raw("deep_file"),
                ty_ident: parse_id_maybe_raw("DeepFile"),
            }],
            dirs: Vec::new(),
        };
        let root = FolderNode {
            fields: vec![ModuleField {
                mod_ident: parse_id_maybe_raw("alpha"),
                ty_ident: parse_id_maybe_raw("Alpha"),
            }],
            dirs: vec![(parse_id_maybe_raw("nested_dir"), child)],
        };
        let vis: syn::Visibility = syn::parse_str("pub(crate)").expect("vis");
        let crate_path = quote! { crate::tree };
        let rendered = render_struct(
            &root,
            &parse_id_maybe_raw("Tree"),
            &crate_path,
            &quote! {},
            &vis,
        )
        .to_string();
        assert!(rendered.contains("pub (crate) mod nested_dir"));
        assert!(rendered.contains("pub (crate) struct NestedDir"));
        assert!(
            rendered.contains("deep_file : crate :: tree :: nested_dir :: deep_file :: DeepFile")
        );
        assert!(rendered.contains("alpha : crate :: tree :: alpha :: Alpha"));
    }

    #[test]
    fn parse_id_maybe_raw_handles_keywords() {
        assert_eq!(parse_id_maybe_raw("demo").to_string(), "demo");
        assert_eq!(parse_id_maybe_raw("type").to_string(), "r#type");
    }

    #[test]
    fn folder_iso_struct_writes_flat_and_recursive_outputs() {
        let _guard = env_guard();
        let base = calling_crate_dir();
        let from_crate = "target/rsmack-utils-unittest";
        let widgets = base.join(from_crate).join("src").join("widgets");
        let panels = widgets.join("panels");
        std::fs::create_dir_all(&panels).expect("create dirs");
        std::fs::write(widgets.join("button.rs"), "").expect("write button");
        std::fs::write(panels.join("sidebar.rs"), "").expect("write sidebar");

        let out = tempdir("out-dir-flat");
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("OUT_DIR", &out) };

        folder_iso_struct()
            .name("Widgets")
            .pre(&quote! { #[derive(Debug)] })
            .from_crate(from_crate)
            .folder("widgets")
            .log_enabled(true)
            .call();
        let flat = std::fs::read_to_string(out.join("Widgets.rs")).expect("read flat output");
        assert!(flat.contains("button : crate :: widgets :: button :: Button"));
        // Flat mode: the directory is mirrored as a plain field
        assert!(flat.contains("panels : crate :: widgets :: panels :: Panels"));

        folder_iso_struct()
            .name("WidgetsTree")
            .pre(&quote! {})
            .from_crate(from_crate)
            .folder("widgets")
            .vis("pub(crate)")
            .recursive(true)
            .call();
        let nested =
            std::fs::read_to_string(out.join("WidgetsTree.rs")).expect("read nested output");
        assert!(nested.contains("pub (crate) mod panels"));
        assert!(nested.contains("pub (crate) struct Panels"));
        assert!(nested.contains("sidebar : crate :: widgets :: panels :: sidebar :: Sidebar"));

        std::fs::remove_dir_all(&out).ok();
        std::fs::remove_dir_all(base.join(from_crate)).ok();
    }

    #[test]
    #[should_panic(expected = "Invalid `vis` value")]
    fn folder_iso_struct_panics_on_invalid_vis() {
        let _guard = env_guard();
        let out = tempdir("out-dir-invalid-vis");
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("OUT_DIR", &out) };
        folder_iso_struct()
            .name("Nope")
            .pre(&quote! {})
            .from_crate("target/nope")
            .folder("nope")
            .vis("definitely not a visibility")
            .call();
    }

    #[test]
    fn generate_file_writes_into_out_dir() {
        let _guard = env_guard();
        let out = tempdir("out-dir-write");
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("OUT_DIR", &out) };
        generate_file("demo.rs", b"struct Demo;");
        let written = std::fs::read_to_string(out.join("demo.rs")).expect("read generated");
        assert_eq!(written, "struct Demo;");
        std::fs::remove_dir_all(&out).ok();
    }

    #[test]
    #[should_panic(expected = "OUT_DIR not set")]
    fn generate_file_panics_without_out_dir() {
        let _guard = env_guard();
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::remove_var("OUT_DIR") };
        generate_file("demo.rs", b"struct Demo;");
    }

    #[test]
    #[should_panic(expected = "Failed to create the generated file")]
    fn generate_file_panics_when_creation_fails() {
        let _guard = env_guard();
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("OUT_DIR", "/dev/null") };
        generate_file("demo.rs", b"struct Demo;");
    }

    #[test]
    fn package_src_folder_appends_src() {
        let _guard = env_guard();
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("CARGO_MANIFEST_DIR", "/tmp/rsmack-fake-crate") };
        let src = package_src_folder();
        assert_eq!(src, PathBuf::from("/tmp/rsmack-fake-crate/src"));
    }

    #[test]
    #[should_panic(expected = "CARGO_MANIFEST_DIR not set")]
    fn package_src_folder_panics_without_env() {
        let _guard = env_guard();
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::remove_var("CARGO_MANIFEST_DIR") };
        let _ = package_src_folder();
    }
}
