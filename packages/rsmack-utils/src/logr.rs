use proc_macro2::Span;

#[derive(bon::Builder)]
#[builder(on(String, into))]
pub struct Logr {
    prefix: String,
}
impl Logr {
    pub fn abort(&self, span: Span, msg: &str) -> ! {
        proc_macro_error::abort!(span, format!("[{}] {}", self.prefix, msg))
    }
}
