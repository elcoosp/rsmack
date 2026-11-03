//! [`folder_iso_struct`](attr.folder_iso_struct.html) — mirror a folder of Rust modules
//! into a struct, **inline at expansion time**.
//!
//! One attribute, zero build script: the folder is scanned when the macro expands and
//! the mirrored struct is emitted directly (nested modules with `recursive = true`).
//! Every generated item carries the annotated struct's attributes and visibility (or
//! the explicit `vis` / `derives` options).
//!
//! ```rust,ignore
//! use rsmack_fs::folder_iso_struct;
//!
//! #[folder_iso_struct(folder = "levels", vis = "pub(crate)", recursive = true, derives("Debug"))]
//! pub struct Levels {
//!     // The declared fields are a sketch only: the mirror is generated from the folder.
//! }
//! ```

use rsmack_megamac::megamac;

mod impls;

megamac!(kind = Attr, name = folder_iso_struct, doc = "readme");
