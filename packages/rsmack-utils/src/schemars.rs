//! Schemas helpers for crates relying on [schemars](https://docs.rs/schemars)
//!
/// Implement `::schemars::JsonSchema` with a plain `{"type": <literal>}` schema.
///
/// ```
/// use rsmack_utils::impl_json_schema;
///
/// struct UserId;
/// impl_json_schema!(UserId, "string");
///
/// let schema: schemars::Schema = schemars::SchemaGenerator::default()
///     .into_root_schema_for::<UserId>();
/// let value = serde_json::to_value(&schema).unwrap();
/// assert_eq!(value["type"], "string");
/// ```
#[macro_export]
macro_rules! impl_json_schema {
    ($id:ident, "string") => {
        impl_json_schema! { @simple_ty $id, "string" }
    };
    (@simple_ty $id:ident, $ty:literal) => {
        impl ::schemars::JsonSchema for $id {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                stringify!($id).into()
            }

            fn json_schema(_generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                ::schemars::json_schema!({
                    "type": $ty,
                })
            }
        }
    };
}
