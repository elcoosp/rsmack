//! Some doc utils (not edoc related)

/// `cl` stands for code link
/// ```
/// use rsmack_utils::cl;
/// let result = cl!(proc_macro_error2::abort);
/// assert_eq!(result, "[`proc_macro_error2::abort`]");
/// let result_macro = cl!(proc_macro_error2::abort!);
/// assert_eq!(result_macro, "[`proc_macro_error2::abort!`]");
/// ```
#[macro_export]
macro_rules! cl {
    ($id:ident!) => {
        concat!(stringify!($id), "!")
    };
    ($($rest:ident)::*!) => {
        concat!("[`", cl!($($rest)*!), "`]")
    };
    ($fst:ident $($id:ident)*!) => {
        concat!(stringify!($fst), "::", cl!($($id)*!))
    };
    ($id:ident) => {
        stringify!($id)
    };
    ($($rest:ident)::*) => {
        concat!("[`", cl!($($rest)*), "`]")
    };
    ($fst:ident $($id:ident)*) => {
        concat!(stringify!($fst), "::", cl!($($id)*))
    };
}
