//! Constants consumed by the edoc integration tests (docs are generated from them).
//!
//! This file is not compiled: the `#[edoc]` macro reads it at expansion time.

pub const APP: &str = "client";
pub const VERSION: &str = "1.0.0";
pub const VERBOSE: bool = true;
pub const MAGIC: &[u8] = b"\x01\x02";
