use proc_macro2::Span;

#[derive(bon::Builder)]
#[builder(on(String, into))]
pub struct Logr {
    prefix: String,
}
impl Logr {
    /// Call [proc_macro_error::abort!] with [Self::prefix]
    pub fn abort(&self, span: Span, msg: &str) -> ! {
        proc_macro_error::abort!(span, format!("[{}] {}", self.prefix, msg))
    }
    /// Create a [Logr] with [std::module_path!] prepend to [Self::prefix]
    pub fn new_module_path(prefix: &str) -> Self {
        let module_path = std::module_path!();
        Self::builder()
            .prefix(format!("{module_path}::{}", prefix))
            .build()
    }
}
