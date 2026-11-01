//! trybuild UI tests: snapshot the exact rustc output of every error path.
//!
//! The snapshots pair with the `Logr` prefix system for stable, grep-able diagnostics:
//! any accidental change of a user-facing message shows up in review.
//!
//! trybuild compiles every case inside a scratch project
//! (`target/tests/trybuild/<crate>`), which does not carry this crate's `src/` fixtures.
//! The file-reading macros (`edoc`, `megamac`) resolve fixtures from
//! `CARGO_MANIFEST_DIR`, so the fixture files are seeded into the scratch project before
//! the cases run. When trybuild changes its layout, the intended errors stop firing and
//! the snapshots fail loudly instead of silently.

use std::path::PathBuf;

/// Seed the fixture files into the trybuild scratch project (best effort: the committed
/// `src/` copies are used when the layout matches, ignored otherwise).
fn seed_scratch_fixtures() {
    let Some(scratch) = scratch_project_dir() else {
        return;
    };
    let write = |relative: &str, content: &str| {
        let path = scratch.join(relative);
        if std::fs::create_dir_all(path.parent().expect("parent dir")).is_ok() {
            let _ = std::fs::write(path, content);
        }
    };
    write("src/consts.rs", "pub const APP: &str = \"ui-tests\";\n");
    write("src/impls/broken.rs", "this is not rust at all $$$\n");
}

/// Best-effort discovery of the trybuild scratch project dir:
/// `<workspace>/target/tests/trybuild/<crate name>`.
fn scratch_project_dir() -> Option<PathBuf> {
    let crate_name = env!("CARGO_PKG_NAME");
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    Some(
        workspace
            .join("target")
            .join("tests")
            .join("trybuild")
            .join(crate_name),
    )
}

#[test]
fn ui() {
    seed_scratch_fixtures();
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
