

# RSMACK - Rust Macro Enhancement Utilities

RSMACK is a collection of utilities designed to enhance procedural macro creation in Rust. It provides tools for generating documentation, handling file structures, type wrapping, and macro generation with proper error handling.

## Crates

### 1. megamac
**Procedural macro generator with automatic documentation and error handling**

The `megamac` crate provides a meta-macro that generates different types of procedural macros (function-like, attribute, derive) with automatic documentation generation and proper error handling.

#### How It Works
Under the hood, `megamac` generates a procedural macro that:
- Automatically includes `#[proc_macro_error]` for enhanced error handling
- Sets up the appropriate `#[proc_macro]`, `#[proc_macro_attribute]`, or `#[proc_macro_derive]` attribute
- Delegates execution to your implementation module

The generated macro structure:
```rust
#[proc_macro_error]
#[proc_macro] // or #[proc_macro_attribute]/#[proc_macro_derive]
pub fn your_macro_name(args: TokenStream) -> TokenStream {
    rsmack_utils::exec::call_func_impls_with_args!(your_macro_name, args)
}
```

#### Implementation Requirements
You must provide an implementation module with the following structure:

**For function-like macros:**
```rust
// In impls/your_macro_name.rs
pub fn exec(args: Args, env: ExecEnv) -> TokenStream {
    // Your macro implementation here
    // Returns TokenStream
}
```

**For attribute macros (with receiver):**
```rust
// In impls/your_attribute_macro.rs
pub fn exec(args: Args, item: ItemStruct, env: ExecEnv) -> TokenStream {
    // Your macro implementation here
    // 'item' contains the struct this attribute is applied to
    // Returns TokenStream
}
```

#### Features:
- **Automatic Boilerplate**: Generates proper procedural macro attributes and error handling
- **Implementation Delegation**: Routes execution to your custom implementation module
- **Multiple Macro Types**: Supports function-like, attribute, and derive macros
- **Field Documentation**: Automatically documents macro arguments and their types
- **Error Handling**: Built-in error handling with `proc_macro_error2`

#### Usage Example:
```rust
use megamac::megamac;

// Generate a function-like procedural macro
megamac! {
    kind = Func,
    name = my_function_macro,
}

// Generate an attribute macro with struct receiver
megamac! {
    kind = Attr,
    name = my_attribute_macro,
    receiver = ItemStruct,  // The type this attribute applies to
}

// Generate a derive macro
megamac! {
    kind = Derive,
    name = MyDeriveMacro,
}
```

#### Implementation Structure
After generating with `megamac`, your crate should have this structure:
```
your_macro_crate/
├── src/
│   ├── lib.rs              # Contains the generated macro
│   └── impls/
│       ├── your_macro_name.rs # Your implementation
│       └── mod.rs          # Module declarations
```

This structure ensures clean separation between the macro boilerplate and your actual implementation logic.

### 2. seanum
**SeaORM enum generator with automatic attribute and derive injection**

The `seanum` crate provides a procedural macro that transforms a simple Rust enum into a fully-featured SeaORM active enum with database mapping capabilities. It automatically adds the necessary attributes, derives, and implementations to make the enum compatible with SeaORM's entity system.

#### How It Works
The `seanum` macro processes an enum definition and:
- Adds necessary imports for SeaORM, serde, and fake
- Adds a derive macro with various traits for database compatibility
- Adds a `#[sea_orm(...)]` attribute to the enum with database mapping information
- Adds `#[sea_orm(string_value = "...")]` attributes to each variant
- Converts the enum name to snake_case for database naming conventions

#### Features:
- **Automatic SeaORM Integration**: Adds all necessary attributes and derives for SeaORM compatibility
- **Database Mapping**: Maps enum variants to their string representations in the database
- **Serialization Support**: Automatically adds serde serialization and deserialization
- **Test Data Generation**: Includes fake data generation support for testing
- **Naming Convention Handling**: Automatically converts enum names to snake_case for database use

#### Usage Example:
```rust
use rsmack::seanum;

#[seanum(rs_type = String, db_type = "Enum")]
pub enum SwitchAction {
    SendEmail,
    SendSms,
}
```

This expands to:
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

#### Parameters:
- `rs_type` (Ident): The Rust type that represents the enum in the database (e.g., `String`, `i32`)
- `db_type` (LitStr): The database type for the enum (e.g., `"Enum"`, `"Text"`)

### 3. edoc
**Enhanced documentation macro with constant evaluation**

The `edoc` macro generates documentation for struct fields by concatenating string literals and constants at compile time, enabling dynamic documentation generation.

#### Features:
- **Constant Evaluation**: Resolves constants from specified modules
- **String Concatenation**: Combines multiple string literals and constants
- **Type-Safe**: Ensures all referenced constants exist and are of supported types
- **Clean Syntax**: Simple attribute-based syntax

#### Usage Example:
```rust
// In constants.rs
const APP_NAME: &str = "MyApplication";
const VERSION: &str = "1.0.0";
const DESCRIPTION: &str = "A fantastic application";

// In your struct
use edoc::edoc;

#[edoc(from = constants)]
struct AppConfig {
    #[edoc(("Application: ", APP_NAME, " v", VERSION))]
    name: String,

    #[edoc(("Description: ", DESCRIPTION))]
    description: String,

    #[edoc(("Built with Rust version: ", env!("RUSTC_VERSION")))]
    rust_version: String,
}
```

This expands to:
```rust
struct AppConfig {
    #[doc = "Application: MyApplication v1.0.0"]
    name: String,

    #[doc = "Description: A fantastic application"]
    description: String,

    #[doc = "Built with Rust version: rustc 1.70.0"]
    rust_version: String,
}
```

### 4. folder_iso_struct
**Compile-time folder structure mirroring**

This macro generates structs that mirror folder structures at compile time, enabling type-safe access to file contents and metadata.

#### Features:
- **Type-Safe File Access**: Compile-time validation of file existence
- **Folder Structure Mirroring**: Automatically creates structs matching directory structures
- **Build-Time Generation**: Processes folder structure during compilation
- **Flat Directory Support**: Designed for flat directories (no subdirectories)

#### Usage Example:
```rust
use folder_iso_struct::folder_iso_struct;

#[folder_iso_struct(
    from_crate = "my_crate",
    folder = "templates"
)]
struct Templates;

// The macro generates a `generate()` method that creates the structure
// at compile time based on your templates folder
```

### 5. wrap
**Struct field type wrapper**

The `wrap` macro automatically wraps struct field types with a specified wrapper type, useful for adding functionality like validation, logging, or transformation.

#### Features:
- **Type Wrapping**: Automatically wraps field types with specified wrapper
- **Flexible Type Support**: Works with path types, slices, tuples, and arrays
- **Pre-Serde Processing**: Designed to be used before serde derives
- **Error Reporting**: Clear error messages for unsupported types

#### Usage Example:
```rust
use wrap::wrap;

// Wrap all fields with Option<T>
#[wrap(with = Option)]
struct MyStruct {
    name: String,
    count: i32,
    tags: Vec<String>,
}

// This expands to:
struct MyStruct {
    name: Option<String>,
    count: Option<i32>,
    tags: Option<Vec<String>>,
}
```

## Installation

Add the desired crates to your `Cargo.toml`:

```toml
[dependencies]
rsmack-megamac = "0.7"
rsmack-seanum = "0.7"
rsmack-edoc = "0.7"
rsmack-fs = "0.7"
rsmack-wrap = "0.7"
rsmack-utils = "0.7"  # Shared utilities
```

## Key Features Across Crates

### Error Handling
All macros use `proc_macro_error2` for improved error reporting with span information.

### Documentation Generation
Automatic documentation generation with support for constant evaluation and string concatenation.

### Type Safety
Compile-time validation of types, constants, and file structures.

### Build-Time Processing
Leverages Rust's build system for processing file structures and generating type-safe accessors.

## Common Use Cases

1. **Macro Development**: Use `megamac` to quickly bootstrap new procedural macros with proper documentation
2. **Database Enums**: Use `seanum` to create SeaORM-compatible enums with minimal boilerplate
3. **Configuration Documentation**: Use `edoc` to generate documentation from constants for configuration structs
4. **Asset Management**: Use `folder_iso_struct` for type-safe access to static assets
5. **Validation Wrappers**: Use `wrap` to automatically add validation wrappers to struct fields

## Requirements

- Rust 1.91 or later
- Procedural macro support
- `syn`, `quote`, `darling` dependencies (included)
