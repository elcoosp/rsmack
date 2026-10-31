//! Some doc utils (not edoc related)

/// `cl` stands for code link
/// ```
/// use rsmack_utils::cl;
/// assert_eq!(cl!(syn::Error), "[`syn::Error`]");
/// assert_eq!(cl!(syn::Error!), "[`syn::Error!`]");
/// assert_eq!(cl!(Debug), "Debug");
/// ```
#[macro_export]
macro_rules! cl {
    ($id:ident!) => {
        concat!(stringify!($id), "!")
    };
    ($($id:ident)::*!) => {
        concat!("[`", cl!($($id)*!), "`]")
    };
    ($fst:ident $($id:ident)*!) => {
        concat!(stringify!($fst), "::", cl!($($id)*!))
    };
    ($id:ident) => {
        stringify!($id)
    };
    ($($id:ident)::*) => {
        concat!("[`", cl!($($id)*), "`]")
    };
    ($fst:ident $($id:ident)*) => {
        concat!(stringify!($fst), "::", cl!($($id)*))
    };
}
