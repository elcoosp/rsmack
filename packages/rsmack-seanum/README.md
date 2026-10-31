# seanum Macro Documentation

## Overview

The `seanum` procedural macro transforms a simple Rust enum into a fully-featured SeaORM
active enum with database mapping capabilities. It automatically adds the necessary
attributes, derives, and implementations to make the enum compatible with SeaORM's entity
system.

## Usage

Apply the `#[seanum]` attribute to an enum definition with the required parameters:

```rust
use rsmack_seanum::seanum;

#[seanum(rs_type = "String", db_type = "Enum")]
pub enum SwitchAction {
    SendEmail,
    SendSms,
}
```

## Parameters

The `seanum` macro accepts the following parameters:

- `rs_type` (Ident): The Rust type that represents the enum in the database (e.g., `"String"`, `"i32"`).
- `db_type` (LitStr): The database type for the enum (e.g., `"Enum"`, `"Text"`). With
  SeaORM 2, string columns require an explicit length, e.g.
  `"String(sea_orm::sea_query::StringLen::None)"`.

## Transformation

When applied, the macro transforms the input enum by:

1. Adding a derive macro with various traits:
   - `Clone` - Enables cloning of the enum
   - `Dummy` - Enables generation of fake data for testing
   - `Debug` - Enables debug formatting
   - `PartialEq` - Enables partial equality comparison
   - `EnumIter` - Enables iteration over enum variants
   - `DeriveActiveEnum` - SeaORM's derive for active enums
   - `Eq` - Enables full equality comparison
   - `Serialize` - Enables serialization
   - `Deserialize` - Enables deserialization
2. Adding a `#[sea_orm(...)]` attribute to the enum with:
   - `rs_type` - The Rust type for the enum in the database
   - `db_type` - The database type for the enum
   - `enum_name` - The snake_case version of the enum name for database use
3. Adding `#[sea_orm(string_value = "...")]` attributes to each variant with the string
   representation of the variant name

## Example

### Input:

```rust
use rsmack_seanum::seanum;

#[seanum(rs_type = "String", db_type = "Enum")]
pub enum SwitchAction {
    SendEmail,
    SendSms,
}
```

### Output:

```rust,ignore
#[derive(
    Clone, ::fake::Dummy, Debug, PartialEq, ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum, Eq, ::serde::Serialize, ::serde::Deserialize, Hash
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "switch_action")]
pub enum SwitchAction {
    #[sea_orm(string_value = "SendEmail")]
    SendEmail,
    #[sea_orm(string_value = "SendSms")]
    SendSms,
}
```

## Notes

- The macro converts the enum name to snake_case for the database name (e.g.,
  `SwitchAction` becomes `switch_action`)
- Each variant is mapped to its string representation in the database
- The generated enum is fully compatible with SeaORM's entity system and can be used in
  database operations
- The macro requires the `megamac` framework for proper execution

## Dependencies

This macro requires the following dependencies in consumer crates:

- `sea-orm` - For the `DeriveActiveEnum` trait and related attributes
- `serde` - For serialization and deserialization
- `fake` - For generating fake data in tests
- `sea-orm-migration` - Optionally, for database migration support
