//! End-to-end test of the `folder_iso_struct` attribute macro.

use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(from_crate = "target/rsmack-fs-it", folder = "levels")]
struct Levels {
    // The macro only requires named fields; the mirrored struct is generated
    // into `OUT_DIR` by `generate()` below.
    #[expect(dead_code)]
    alpha: (),
}

#[test]
fn generate_mirrors_the_folder_into_out_dir() {
    // Prepare the folder to mirror, under `target/` (gitignored)
    let base = rsmack_utils::fs::calling_crate_dir();
    let levels_dir = base.join("target/rsmack-fs-it/src/levels");
    std::fs::create_dir_all(&levels_dir).expect("create levels dir");
    std::fs::write(levels_dir.join("alpha.rs"), "").expect("write alpha");
    std::fs::write(levels_dir.join("beta_gamma.rs"), "").expect("write beta_gamma");
    std::fs::write(levels_dir.join("mod.rs"), "").expect("write mod");

    let out = std::env::temp_dir().join("rsmack-fs-it-out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("create out dir");
    // SAFETY: only test in this binary mutating the environment
    unsafe { std::env::set_var("OUT_DIR", &out) };

    Levels::generate();

    let generated = std::fs::read_to_string(out.join("Levels.rs")).expect("read generated");
    assert!(generated.contains("alpha : crate :: levels :: alpha :: Alpha"));
    assert!(generated.contains("beta_gamma : crate :: levels :: beta_gamma :: BetaGamma"));
    // mod.rs entries are skipped
    assert!(!generated.contains("mod :"));

    std::fs::remove_dir_all(&out).ok();
}
