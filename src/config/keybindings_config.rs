//! This config file contains keybindings for actions within the app
//! Note that ***THESE ARE BACKEND SPECIFIC*** - the current keybindings will *only* work with [`imgui_winit_support`]
use std::fmt::{Display, Formatter};

pub type KeyCode = imgui_winit_support::winit::event::VirtualKeyCode;

#[derive(Debug)]
pub struct KeyBinding<'a>
{
    pub shortcut: &'a [KeyCode],
    pub shortcut_i32: &'a [i32],
    pub shortcut_text: &'a str,
}

impl<'a> Display for KeyBinding<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?})", self.shortcut_text, self.shortcut_i32)
    }
}

macro_rules! keybind {
    ($key:expr) => {
        crate::config::keybindings_config::KeyBinding {
            shortcut: &[$key],
            shortcut_i32: &[$key as i32],
            shortcut_text: stringify!($key),
        }
    };
    ($key:expr, $($modifiers:expr),+) => {
        crate::config::keybindings_config::KeyBinding {
            shortcut: &[$key, $($modifiers),*],
            shortcut_i32: &[$key as i32, $($modifiers as i32),+],
            shortcut_text: stringify!($($modifiers + ) *$key),
        }
    };
}

/// Normal keybindings, not specific to any super-special actions or windows
pub mod standard {
    use crate::config::keybindings_config::KeyBinding;
    use imgui_winit_support::winit::event::VirtualKeyCode::*;

    /// Toggles the visibility of the [imgui] metrics window (see [imgui::Ui::show_metrics_window()])
    pub const KEY_TOGGLE_METRICS_WINDOW: KeyBinding = keybind!(F3);
    /// Toggles the visibility of the [imgui] demo window (see [imgui::Ui::show_demo_window()])
    pub const KEY_TOGGLE_DEMO_WINDOW: KeyBinding = keybind!(F1);
    /// Toggles the visibility of the [UiManagers] window
    pub const KEY_TOGGLE_UI_MANAGERS_WINDOW: KeyBinding = keybind!(F6);

    /// (kinda) Dummy keybinding for exiting the app
    ///
    /// Not really necessary as the OS should send the quit signal anyway, but we might as well have it just in case
    pub const KEY_EXIT_APP: KeyBinding = keybind!(F4, LAlt);
}
