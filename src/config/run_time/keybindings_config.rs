//! This config file contains keybindings for actions within the app
//! Note that ***THESE ARE BACKEND SPECIFIC*** - the current keybindings will *only* work with [`imgui_winit_support`]
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

pub type KeyCode = imgui_winit_support::winit::event::VirtualKeyCode;

/// Config struct that holds keybinding values
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Toggles the visibility of the [imgui] metrics window (see [imgui::Ui::show_metrics_window()])
    pub toggle_metrics_window: KeyBinding,
    /// Toggles the visibility of the [imgui] demo window (see [imgui::Ui::show_demo_window()])
    pub toggle_demo_window: KeyBinding,
    /// Toggles the visibility of the [UiManagers] window
    pub toggle_ui_managers_window: KeyBinding,

    /// (kinda) Dummy keybinding for exiting the app
    ///
    /// Not really necessary as the OS should send the quit signal anyway, but we might as well have it just in case
    pub exit_app: KeyBinding,
}

/// Represents a keybinding (a key, and possible modifiers)
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub shortcut: KeyCode,
    pub modifier_ctrl: bool,
    pub modifier_alt: bool,
    pub modifier_shift: bool,
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.modifier_ctrl {
            f.write_str("Ctrl + ")?
        }
        if self.modifier_alt {
            f.write_str("Alt + ")?
        }
        if self.modifier_shift {
            f.write_str("Shift + ")?
        }
        write!(f, "{:?}", self.shortcut)
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            toggle_metrics_window: KeyBinding {
                shortcut: KeyCode::F3,
                modifier_ctrl: false,
                modifier_alt: false,
                modifier_shift: false,
            },
            toggle_demo_window: KeyBinding {
                shortcut: KeyCode::F1,
                modifier_ctrl: false,
                modifier_alt: false,
                modifier_shift: false,
            },
            toggle_ui_managers_window: KeyBinding {
                shortcut: KeyCode::F6,
                modifier_ctrl: false,
                modifier_alt: false,
                modifier_shift: false,
            },
            exit_app: KeyBinding {
                shortcut: KeyCode::F4,
                modifier_ctrl: false,
                modifier_alt: true,
                modifier_shift: false,
            },
        }
    }
}

impl KeybindingsConfig {
    /// Creates a new (default) keybindings config
    pub fn new() -> Self {
        Self::default()
    }
}