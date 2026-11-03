//! End-to-end test of the `folder_iso_struct` attribute macro (inline emission).
//!
//! The macro scans `$CARGO_MANIFEST_DIR/src/levels` at expansion time, so the fixture
//! folder lives in this crate's `src/`; the mirrored module tree is declared below so
//! the generated references resolve.

use rsmack_fs::folder_iso_struct;

/// The real module tree mirrored by the generated struct.
pub mod levels {
    pub mod beta_gamma {
        #[derive(Debug)]
        pub struct BetaGamma;
    }
    pub mod button {
        #[derive(Debug)]
        pub struct Button;
    }
}

#[folder_iso_struct(folder = "levels", vis = "pub", derives("Debug"))]
pub struct Levels {
    // The declared fields are a sketch only: the mirror is generated from the folder.
    #[expect(dead_code)]
    alpha: (),
}

#[test]
fn macro_emits_the_mirrored_struct_inline() {
    // The mirror is a real, usable struct: the fixture modules exist above.
    let levels_mirror = Levels {
        button: levels::button::Button,
        beta_gamma: levels::beta_gamma::BetaGamma,
    };
    // mod.rs entries are skipped: no `mod` field.
    let debug = format!("{levels_mirror:?}");
    assert!(debug.contains("Levels"));
    assert!(debug.contains("button"));
    assert!(debug.contains("beta_gamma"));
    assert!(!debug.contains("mod"));
}
