//! This module contains configuration options for the entire application, separated out into further sub-modules
#[macro_use]
pub mod ui_config;
#[macro_use]
pub mod tracing_config;
#[macro_use]
pub mod program_config;
#[macro_use]
pub mod resources_config;
#[macro_use]
pub mod keybindings_config;

/// Macro that generates a config flag with a given [name] and [value]. Field will be `static` to ensure no "expression always true" warnings happen
#[macro_export]
macro_rules! flag {
    ($name:ident, $value:expr, $documentation:literal) => {
        #[doc=$documentation]
        pub static $name: bool = $value;
    };
}
/// Macro that generates a config string with a given [name] and [value]. Field will be `static`
#[macro_export]
macro_rules! string {
    ($name:ident, $value:expr, $documentation:literal) => {
        #[doc=$documentation]
        pub static $name: bool = $value;
    };
}
// /// Macro that generates a config flag with a given [name] and [value]. Same as [flag] but generates a `const` field not a `static` one
// #[macro_export]
// macro_rules! const_flag {
//     ($name:ident, $value:expr) => {pub const $name:bool = $value;};
// }