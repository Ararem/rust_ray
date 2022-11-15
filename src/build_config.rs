//! Contains compile-time configuration options

/// Macro that generates a config flag with a given [name] and [value]. Field will be `static` to ensure no "expression always true" warnings happen
macro_rules! flag {
    ($name:ident, $value:expr) => {pub static $name:bool = $value;};
}
// /// Macro that generates a config flag with a given [name] and [value]. Same as [flag] but generates a `const` field not a `static` one
// macro_rules! const_flag {
//     ($name:ident, $value:expr) => {pub const $name:bool = $value;};
// }

pub(crate) mod tracing{
    flag!(ENABLE_UI_TRACE, false);
}