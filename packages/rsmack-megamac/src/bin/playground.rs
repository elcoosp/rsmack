//! CLI playground printing the expansion of a rsmack-based macro.
//!
//! ```text
//! cargo run -p rsmack-megamac --bin playground -- --kind Attr --name edoc
//! ```
//!
//! Run inside any rsmack-based crate: the impl file is resolved from
//! `--src-dir` (defaults to `$CARGO_MANIFEST_DIR/src`), exactly like `megamac!` does
//! at compile time.
//!
//! The proc-macro crate type cannot export non-macro items, so the playground
//! `#[path]`-includes the `megamac` implementation module instead of going through
//! the library target.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[path = "../impls/megamac.rs"]
mod megamac;

use rsmack_utils::megamac::ExecEnv;

/// Render the expansion of the `megamac!` invocation described by the arguments.
///
/// # Errors
/// Returns a user-facing message when the `megamac!` call itself would fail (unknown
/// kind, unreadable or unparsable impl file, missing args struct, ...).
fn expand(
    kind: &str,
    name: &str,
    src_folder: &Path,
    impls: Option<&str>,
    args: Option<&str>,
    doc: Option<&str>,
) -> Result<String, String> {
    let ident = |value: &str| -> Result<syn::Ident, String> {
        syn::parse_str::<syn::Ident>(value)
            .map_err(|_| format!("`{value}` is not a valid Rust identifier"))
    };
    let args = megamac::Args {
        kind: ident(kind)?,
        name: ident(name)?,
        impls: impls.map(ident).transpose()?,
        arguments: args.map(ident).transpose()?,
        doc: doc.map(ident).transpose()?,
    };
    let env = ExecEnv::builder(format!("playground::{name}")).build();
    match megamac::run(&args, &env, src_folder) {
        Ok(tokens) => Ok(tokens.to_string()),
        Err(error) => Err(error.to_string()),
    }
}

/// Parsed CLI options.
struct Opts {
    values: Vec<(String, String)>,
}

impl Opts {
    fn get(&self, name: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let mut values: Vec<(String, String)> = Vec::new();
    while let (Some(flag), Some(value)) = (argv.next(), argv.next()) {
        let Some(flag) = flag.strip_prefix("--") else {
            eprintln!("error: unexpected argument `{flag}`, expected `--<option> <value>`");
            return usage();
        };
        values.push((flag.to_string(), value));
    }
    let opts = Opts { values };

    for required in ["kind", "name"] {
        if opts.get(required).is_none() {
            eprintln!("error: missing required option `--{required}`");
            return usage();
        }
    }
    let kind = opts.get("kind").expect("checked above");
    let name = opts.get("name").expect("checked above");
    let src_dir = match opts.get("src-dir") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default()).join("src"),
    };

    match expand(
        kind,
        name,
        &src_dir,
        opts.get("impls"),
        opts.get("args"),
        opts.get("doc"),
    ) {
        Ok(expansion) => {
            println!("{expansion}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Print the usage summary and exit with a failure code.
fn usage() -> ExitCode {
    eprintln!(
        "usage: playground --kind <Func|Attr|Derive> --name <macro> [--src-dir <path>] \
         [--impls <ident>] [--args <ident>] [--doc <readme|inline>]"
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src_folder() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    #[test]
    fn expand_renders_the_megamac_entrypoint() {
        let expansion =
            expand("Func", "megamac", &src_folder(), None, None, None).expect("expansion succeeds");
        assert!(expansion.contains("# [proc_macro]"));
        assert!(expansion.contains("pub fn megamac"));
        assert!(expansion.contains("func_entrypoint"));
        assert!(expansion.contains("* `kind`"));
    }

    #[test]
    fn expand_reports_invalid_idents_and_kinds() {
        let error = expand("Func", "not an ident!", &src_folder(), None, None, None)
            .expect_err("invalid ident fails");
        assert!(error.contains("is not a valid Rust identifier"));

        let error = expand("Nope", "demo", &src_folder(), None, None, None)
            .expect_err("unknown kind fails");
        assert!(error.contains("Unsupported MacroKind `Nope`"));
    }

    #[test]
    fn expand_reports_missing_impl_files() {
        let error = expand("Func", "does_not_exist", &src_folder(), None, None, None)
            .expect_err("missing impl file fails");
        assert!(error.contains("Failed to get macro_impl_src of does_not_exist"));
    }

    #[test]
    fn expand_honors_custom_impls_ident() {
        let error = expand(
            "Func",
            "megamac",
            &src_folder(),
            Some("attributes"),
            None,
            None,
        )
        .expect_err("custom impls dir does not exist");
        assert!(error.contains("attributes"));
    }
}
