//! The Rust proc-macro factory, behind one facade.
//!
//! `rsmack` re-exports every shipped macro so a consumer needs a single dependency:
//!
//! ```toml
//! [dependencies]
//! rsmack = "1"                        # everything
//! rsmack = { version = "1", default-features = false, features = ["edoc"] }
//! ```
//!
//! | Feature | Macro | Purpose |
//! | --- | --- | --- |
//! | `edoc` | `#[edoc]` | docs from constants, with const-expression evaluation |
//! | `wrap` | `#[wrap]` | wrap struct field types, per-field overrides |
//! | `seanum` | `#[seanum]` | `SeaORM` active enums with sane defaults + migrations |
//! | `fs` | `#[folder_iso_struct]` | compile-time folder mirroring, inline |
//! | `megamac` | `megamac!` | generate your own documented macros from an `exec` fn |
//!
//! Authoring your own macro with `megamac` additionally requires a direct
//! [`rsmack-utils`](https://docs.rs/rsmack-utils) dependency: the generated entrypoint
//! references its exec drivers, and the `ExecEnv` type is imported from it in `impl`
//! files (proc-macro crates cannot be imported as normal modules).

#[cfg(feature = "edoc")]
pub use rsmack_edoc::edoc;
#[cfg(feature = "fs")]
pub use rsmack_fs::folder_iso_struct;
#[cfg(feature = "megamac")]
pub use rsmack_megamac::megamac;
#[cfg(feature = "seanum")]
pub use rsmack_seanum::seanum;
#[cfg(feature = "wrap")]
pub use rsmack_wrap::wrap;
