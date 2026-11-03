//! [`edoc`](attr.edoc.html) — documentation from constants, with const-expression
//! evaluation.
//!
//! Evaluate `#[edoc((...))]` tuples of string literals, constant references and **const
//! expressions** into real `#[doc]` attributes on struct fields, enum variants and the
//! annotated item itself. The constants file supports literals, arithmetic, casts,
//! `concat!`, `env!`, `option_env!` and const-to-const references resolved against full
//! module paths. Resolution is memoized per file (process-global, mtime-validated), and
//! every broken expression of an item is reported in one pass instead of failing on the
//! first.
//!
//! ```rust,ignore
//! use rsmack_edoc::edoc;
//!
//! mod consts {
//!     pub const APP: &str = "myapp";
//!     pub const MAJOR: i32 = 1;
//!     pub const TAG: &str = concat!("v", MAJOR);
//! }
//!
//! #[edoc(from = self::consts)]
//! struct Client {
//!     #[edoc(("Client ", APP, " tag ", TAG, " build ", 2 * 21))]
//!     name: String,
//! }
//! ```

use rsmack_megamac::megamac;

mod impls;

megamac!(kind = Attr, name = edoc, doc = "readme");
