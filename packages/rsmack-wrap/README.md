# rsmack-wrap

`#[wrap]` — wrap struct field types, per-field overrides.

```rust,ignore
use rsmack_wrap::wrap;

#[wrap(with = Option)]
struct User {
    name: String,                  // becomes Option<String>
    tags: Vec<String>,             // becomes Option<Vec<String>>
    #[wrap(skip)]
    untouched: &'static str,       // stays &'static str
    #[wrap(with = "Box")]
    boxed: String,                 // becomes Box<String>
    #[wrap(with = "Vec", map = "u8")]
    raw: String,                   // becomes Vec<u8> (map replaces the inner type)
}
```

Every field type wraps — paths, slices, tuples, arrays, references, raw pointers, bare
function pointers and trait objects are all legal inside a generic wrapper
(`Option<&str>`, `Box<dyn Trait>`, ...).

See the [workspace README](https://github.com/elcoosp/rsmack) for the full documentation.
