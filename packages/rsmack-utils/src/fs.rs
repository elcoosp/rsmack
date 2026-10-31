//! [rsmack-fs](../rsmack_fs/index.html) related utils

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use bon::builder;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Ident;

/// Get the directory (workspace root) from which we are compiling.
///
/// Walks up from `OUT_DIR` until a directory containing a `Cargo.toml` is found, which
/// makes it robust to custom target directory layouts (`CARGO_TARGET_DIR`, coverage
/// runs, ...). The result is memoized.
///
/// # Panics
/// If no parent directory of `OUT_DIR` contains a `Cargo.toml`.
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

/// Collect the [`ModuleField`]s mirroring a flat folder of rust modules.
///
/// `mod.rs` files are skipped, everything else (files and directories) becomes a field.
///
/// # Panics
/// - If `mods_folder` cannot be read
/// - If a file stem is not valid UTF-8
fn module_fields(mods_folder: &Path) -> Vec<ModuleField> {
    let opts = stringcase::Options {
        separate_before_non_alphabets: true,
        separate_after_non_alphabets: true,
        separators: "_",
        keep: "",
    };
    let entries = std::fs::read_dir(mods_folder).expect("Failed to read the folder to mirror");
    let mut fields = Vec::new();
    for entry in entries {
        let path = entry.expect("Failed to read a folder entry").path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Non UTF-8 or stem-less module file name");
        if stem == "mod" {
            continue;
        }
        fields.push(ModuleField {
            mod_ident: parse_id_maybe_raw(stem),
            ty_ident: parse_id_maybe_raw(&stringcase::pascal_case_with_options(stem, &opts)),
        });
    }
    fields
}

/// Render the final `pub struct` mirroring `crate::<folder>::<module>::<Type>` fields.
fn struct_source(name: &str, folder: &str, pre: &TokenStream, fields: &[ModuleField]) -> String {
    let folder_id = parse_id_maybe_raw(folder);
    let name_id = parse_id_maybe_raw(name);
    let fields_ts = fields.iter().map(|f| {
        let mod_ident = &f.mod_ident;
        let ty_ident = &f.ty_ident;
        let ty_path = quote! { crate::#folder_id::#mod_ident::#ty_ident };
        quote! { #mod_ident: #ty_path }
    });
    quote! {
        #pre
        pub struct #name_id {
            #(#fields_ts),*
        }
    }
    .to_token_stream()
    .to_string()
}

/// Generate a struct which fields match a flat folder of rust modules, each exposing at
/// least a same file named **`PascalCase`** type.
#[builder]
pub fn folder_iso_struct(
    name: &str,
    pre: &TokenStream,
    folder: &str,
    from_crate: &str,
    #[builder(default = false)] log_enabled: bool,
) {
    use build_print::*;

    let ccd = calling_crate_dir();
    let mods_folder_path = ccd.join(from_crate).join("src").join(folder);
    let fields = module_fields(&mods_folder_path);
    if log_enabled {
        for f in &fields {
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
    let output = struct_source(name, folder, pre, &fields);
    generate_file(format!("{name}.rs"), output.as_bytes());
}

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
pub fn package_src_folder() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("src")
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
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
    fn module_fields_skip_mod_rs_and_pascal_case_names() {
        let dir = tempdir("module-fields");
        for file in ["alpha.rs", "beta_gamma.rs", "mod.rs", "snake_case.rs"] {
            std::fs::write(dir.join(file), "").expect("write module file");
        }
        std::fs::create_dir(dir.join("nested_dir")).expect("create nested dir");
        let fields = module_fields(&dir);
        let mut rendered: Vec<(String, String)> = fields
            .into_iter()
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
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    #[should_panic(expected = "Failed to read the folder to mirror")]
    fn module_fields_panic_on_missing_folder() {
        let _ = module_fields(Path::new("/definitely/not/a/folder"));
    }

    #[test]
    #[should_panic(expected = "Non UTF-8 or stem-less module file name")]
    fn module_fields_panic_on_non_utf8_file_name() {
        use std::os::unix::ffi::OsStrExt;
        let dir = tempdir("non-utf8");
        std::fs::create_dir_all(&dir).expect("create dir");
        let bad_name = std::ffi::OsStr::from_bytes(b"\xff\xfe.rs");
        std::fs::write(dir.join(bad_name), b"").expect("write non utf-8 file");
        let _ = module_fields(&dir);
    }

    #[test]
    fn struct_source_renders_fields_with_pre_attrs() {
        let fields = vec![ModuleField {
            mod_ident: parse_id_maybe_raw("alpha"),
            ty_ident: parse_id_maybe_raw("Alpha"),
        }];
        let source = struct_source("Levels", "levels", &quote! { #[derive(Debug)] }, &fields);
        assert!(source.contains("# [derive (Debug)]"));
        assert!(source.contains("pub struct Levels"));
        assert!(source.contains("alpha : crate :: levels :: alpha :: Alpha"));
    }

    #[test]
    fn parse_id_maybe_raw_handles_keywords() {
        assert_eq!(parse_id_maybe_raw("demo").to_string(), "demo");
        assert_eq!(parse_id_maybe_raw("type").to_string(), "r#type");
    }

    #[test]
    fn generate_file_writes_into_out_dir() {
        let _guard = env_guard();
        let out = tempdir("out-dir");
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::set_var("OUT_DIR", &out) };
        generate_file("demo.rs", b"struct Demo;");
        let written = std::fs::read_to_string(out.join("demo.rs")).expect("read generated");
        assert_eq!(written, "struct Demo;");
        // SAFETY: env mutations are serialized by `ENV_LOCK`
        unsafe { std::env::remove_var("OUT_DIR") };
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
