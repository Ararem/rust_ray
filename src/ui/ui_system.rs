//! Module that contains the structs used in the [crate::ui] module
use crate::ui::font_manager::FontManager;
use glium::glutin::event_loop::EventLoop;
use glium::Display;
use imgui::Context;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::dpi::Size;
use imgui_winit_support::WinitPlatform;

/// Struct that encapsulates the UI system components
pub(in crate::ui) struct UiSystem {
    pub backend: UiBackend,
    pub managers: UiManagers,
}

pub(in crate::ui) struct UiBackend {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub imgui_context: Context,
    pub platform: WinitPlatform,
    /// The renderer that renders the current UI system
    pub renderer: Renderer,
}

pub(in crate::ui) struct UiManagers {
    pub font_manager: FontManager,
}

/// Struct used to configure the UI system
#[derive(Debug, Copy, Clone)]
pub(in crate::ui) struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
    pub default_window_size: Size,
}
