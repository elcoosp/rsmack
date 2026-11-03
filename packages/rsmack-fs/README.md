# rsmack-fs

`#[folder_iso_struct]` — mirror a folder of Rust modules into a struct, **inline at
expansion time**.

One attribute, zero build script: the folder is scanned when the macro expands and the
mirrored struct is emitted directly.

```rust,ignore
use rsmack_fs::folder_iso_struct;

#[folder_iso_struct(folder = "levels", vis = "pub(crate)", recursive = true, derives("Debug"))]
pub struct Levels {
    // The declared fields are a sketch only: the mirror is generated from the folder.
}
```

- Files become struct fields; each module file must expose a `PascalCase`-named type.
- `recursive = true` turns subdirectories into nested modules with their own mirrored
  struct.
- The annotated struct's attributes and visibility are carried over to every generated
  item; `vis` overrides the visibility, `derives(...)` injects extra derives.
- The mirror references `crate::<folder>::<mod>::<Type>` — the modules must exist in the
  calling crate.

Need explicit `OUT_DIR` codegen from a build script instead?
[`rsmack-utils::fs`](https://docs.rs/rsmack-utils) ships the
`folder_iso_struct` builder and `generate_file` helpers.

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
