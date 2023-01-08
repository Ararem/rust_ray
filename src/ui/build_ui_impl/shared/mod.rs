//! Module of shared functions used for the UI building

use std::ffi::c_void;
use crate::config::run_time::ui_config::theme::Theme;
use imgui::Ui;
use crate::ui::build_ui_impl::shared::constants::{MISSING_VALUE_TEXT, NULL_POINTER_TEXT};

pub mod constants;
pub mod error_display;
pub mod input;
pub mod menu_utils;
pub mod tree_utils;
pub mod window_utils;

pub fn display_maybe_c_mut_pointer(ui: &Ui, colours: &Theme, maybe_ptr: Option<*mut c_void>) {
    display_maybe_c_const_pointer(ui, colours, maybe_ptr.map(<*mut c_void>::cast_const))
}
pub fn display_maybe_c_const_pointer(ui: &Ui, colours: &Theme, maybe_ptr: Option<*const c_void>) {
    match maybe_ptr {
        None => {
            ui.text_colored(colours.value.missing_value, MISSING_VALUE_TEXT);
        }
        Some(ptr) => {
            display_c_const_pointer(ui, colours, ptr);
        }
    }
}
pub fn display_c_mut_pointer(ui: &Ui, colours: &Theme, ptr: *mut c_void) {
    display_c_const_pointer(ui,colours, ptr.cast_const())
}
pub fn display_c_const_pointer(ui: &Ui, colours: &Theme, ptr: *const c_void) {
    // /// The number of characters that a [usize] requires when formatted in hex
    // /// x2 cause each byte takes 2 hex chars
    // const USIZE_HEX_WIDTH: usize = 2 * core::mem::size_of::<usize>();
    // format!("0X{ptr:0width$X}", ptr = ptr as usize, width = USIZE_HEX_WIDTH)
    if ptr.is_null() {
        ui.text_colored(colours.value.missing_value, NULL_POINTER_TEXT);
    } else {
        ui.text_colored(
            colours.value.number,
            format!("{ptr:#0X}", ptr = ptr as usize),
        );
    }
}
