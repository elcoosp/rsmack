

# seanum Macro Documentation

## Overview

The `seanum` procedural macro transforms a simple Rust enum into a fully-featured SeaORM active enum with database mapping capabilities. It automatically adds the necessary attributes, derives, and implementations to make the enum compatible with SeaORM's entity system.

## Usage

Apply the `#[seanum]` attribute to an enum definition with the required parameters:

```rust
use rsmack_seanum;

#[seanum(rs_type = String, db_type = "Enum")]
pub enum SwitchAction {
    SendEmail,
    SendSms,
}
```

## Parameters

The `seanum` macro accepts the following parameters:

- `rs_type` (Ident): The Rust type that represents the enum in the database (e.g., `String`, `i32`).
- `db_type` (LitStr): The database type for the enum (e.g., `"Enum"`, `"Text"`).

## Transformation

When applied, the macro transforms the input enum by:

1. Adding necessary imports for SeaORM, serde, and fake
2. Adding a derive macro with various traits:
   - `Clone` - Enables cloning of the enum
   - `Dummy` - Enables generation of fake data for testing
   - `Debug` - Enables debug formatting
   - `PartialEq` - Enables partial equality comparison
   - `EnumIter` - Enables iteration over enum variants
   - `DeriveActiveEnum` - SeaORM's derive for active enums
   - `Eq` - Enables full equality comparison
   - `Serialize` - Enables serialization
   - `Deserialize` - Enables deserialization
3. Adding a `#[sea_orm(...)]` attribute to the enum with:
   - `rs_type` - The Rust type for the enum in the database
   - `db_type` - The database type for the enum
   - `enum_name` - The snake_case version of the enum name for database use
4. Adding `#[sea_orm(string_value = "...")]` attributes to each variant with the string representation of the variant name

## Example

### Input:

```rust
use rsmack_seanum;

#[seanum(rs_type = String, db_type = "Enum")]
pub enum SwitchAction {
    SendEmail,
    SendSms,
}
```

### Output:

```rust
use fake::Dummy;
use sea_orm::entity::prelude::*;
use sea_orm_migration::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Dummy, Debug, PartialEq, EnumIter, DeriveActiveEnum, Eq, Serialize, Deserialize,
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

- The macro automatically converts the enum name to snake_case for the database name (e.g., `SwitchAction` becomes `switch_action`)
- Each variant is mapped to its string representation in the database
- The generated enum is fully compatible with SeaORM's entity system and can be used in database operations
- The macro requires the `megamac` framework for proper execution

## Dependencies

This macro requires the following dependencies:

- `sea-orm` - For the `DeriveActiveEnum` trait and related attributes
- `sea-orm-migration` - For database migration support
- `serde` - For serialization and deserialization
- `fake` - For generating fake data in tests
