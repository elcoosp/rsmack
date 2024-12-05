//! This module expose [Logr] a **proc-macro only** logger wrapper around [proc_macro_error2]
use proc_macro2::Span;
use std::fmt::Display;
#[derive(Debug, bon::Builder)]
#[builder(on(String, into))]
/// Logger around [proc_macro_error2], **only for proc-macros**
pub struct Logr {
    pub prefix: String,
}
macro_rules! emit_msg_with_span {
    ($ident:ident, $ret:ty) => {
        #[doc = concat!("Call [proc_macro_error2::",stringify!($ident),"!] with [`Self::prefix`]")]
        pub fn $ident<M: AsRef<str> + Display>(&self, span: Span, msg: M) -> $ret {
            proc_macro_error2::$ident!(span, self.fmt_msg(msg))
        }
    };
}
macro_rules! emit_msg {
    ($ident:ident, $ret:ty) => {
        #[doc = concat!("Call [proc_macro_error2::",stringify!($ident),"!] with [`Self::prefix`]")]
        pub fn $ident<M: AsRef<str> + Display>(&self, msg: M) -> $ret {
            proc_macro_error2::$ident!(self.fmt_msg(msg))
        }
    };
}
impl Logr {
    fn fmt_msg<M: AsRef<str> + Display>(&self, msg: M) -> String {
        format!("#[{}] {}", self.prefix, msg)
    }
    emit_msg_with_span! {abort, !}
    emit_msg_with_span! {emit_error, ()}
    emit_msg_with_span! {emit_warning, ()}
    emit_msg! {abort_call_site, !}
    emit_msg! {emit_call_site_error, ()}
    emit_msg! {emit_call_site_warning, ()}
}
