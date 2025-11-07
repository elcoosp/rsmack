# RSMACK - Rust Macro Enhancement Utilities

RSMACK is a collection of utilities designed to enhance procedural macro creation in Rust. It provides tools for generating documentation, handling file structures, type wrapping, and macro generation with proper error handling.

## Crates

### 1. megamac
**Procedural macro generator with automatic documentation and error handling**

The `megamac` crate provides a meta-macro that can generate different types of procedural macros (function-like, attribute, derive) with automatic documentation generation and proper error handling.

#### Features:
- **Automatic Documentation**: Generates comprehensive documentation for your macros
- **Error Handling**: Built-in error handling with `proc_macro_error2`
- **Multiple Macro Types**: Supports function-like, attribute, and derive macros
- **Field Documentation**: Automatically documents macro arguments and their types

#### Usage Example:
```rust
use megamac::megamac;

// Generate a function-like procedural macro
megamac! {
    kind = Func,
    name = my_function_macro,
}

// Generate an attribute macro
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

### 2. edoc
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

### 3. folder_iso_struct
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

### 4. wrap
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
2. **Configuration Documentation**: Use `edoc` to generate documentation from constants for configuration structs
3. **Asset Management**: Use `folder_iso_struct` for type-safe access to static assets
4. **Validation Wrappers**: Use `wrap` to automatically add validation wrappers to struct fields

## Requirements

- Rust 1.91 or later
- Procedural macro support
- `syn`, `quote`, `darling` dependencies (included)
