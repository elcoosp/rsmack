#[macro_export]
macro_rules! impl_json_schema {
    ($id:ident, "string") => {
        impl_json_schema!{ @simple_ty $id, "string" }
    };
    (@simple_ty $id:ident, $ty:literal) => {
        impl schemars::JsonSchema for $id {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($id).into()
            }

            fn json_schema(_generator: &mut schemars::SchemaGenerator) -> Schema {
                json_schema!({
                    "type": $ty,
                })
            }
        }
    };
}
