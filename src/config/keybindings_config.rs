//! This config file contains keybindings for actions within the app
//! Note that ***THESE ARE BACKEND SPECIFIC*** - the current keybindings will *only* work with [`imgui_winit_support`]

/// Normal keybindings, not specific to any super-special actions or windows
pub mod standard {
    use imgui_winit_support::winit::event::VirtualKeyCode;

    /// Toggles the visibility of the [imgui] metrics window (see [imgui::Ui::show_metrics_window()])
    pub const TOGGLE_METRICS_WINDOW: i32 = VirtualKeyCode::F3 as i32;
    /// Toggles the visibility of the [imgui] demo window (see [imgui::Ui::show_demo_window()])
    pub const TOGGLE_DEMO_WINDOW: i32 = VirtualKeyCode::F1 as i32;
}