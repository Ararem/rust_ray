//! This config file contains keybindings for actions within the app
//! Note that ***THESE ARE BACKEND SPECIFIC*** - the current keybindings will *only* work with [`imgui_winit_support`]

/// Normal keybindings, not specific to any super-special actions or windows
pub mod standard {
    use imgui_winit_support::winit::event::VirtualKeyCode;

    /// Toggles the visibility of the [imgui] metrics window (see [imgui::Ui::show_metrics_window()])
    pub const KEY_TOGGLE_METRICS_WINDOW: VirtualKeyCode = VirtualKeyCode::F3;
    /// Toggles the visibility of the [imgui] demo window (see [imgui::Ui::show_demo_window()])
    pub const KEY_TOGGLE_DEMO_WINDOW: VirtualKeyCode = VirtualKeyCode::F1;
    /// Toggles the visibility of the [UiManagers] window
    pub const KEY_TOGGLE_UI_MANAGERS_WINDOW: VirtualKeyCode = VirtualKeyCode::F6;
}
