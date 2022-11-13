//! Core module that contains important stuff to make the app work at a low level
//Include the source files
mod program;
mod ui;

//Export them publicly for use elsewhere
pub use program::run_program;
pub use ui::{init,UiConfig,UiSystem};