use std::path::PathBuf;

use color_eyre::eyre;
use glium::{Display, glutin};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use imgui::{Condition, Context, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_winit_support::winit::dpi::Size;
use tracing::{debug_span, instrument, trace, warn};

use crate::config::program_config::{IMGUI_LOG_FILE_PATH, IMGUI_SETTINGS_FILE_PATH};
use crate::log_expr_val;
use crate::program::ui_system::font_manager::FontManager;

pub mod clipboard_integration;
pub mod font_manager;
pub mod docking;

/*
TODO:   Add support for different renderers (glow, glium, maybe d3d12, dx11, wgpu) and backend platforms (winit, sdl2)
        Should probably add an enum selection to [UiConfig]. Also see if it's possible to change hotly or requires a reload
*/

/// Struct that encapsulates the UI system components
pub struct UiSystem {
    pub backend: UiBackend,
    pub managers: UiManagers,
}

pub struct UiBackend {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub imgui_context: Context,
    pub platform: WinitPlatform,
    /// The renderer that renders the current UI system
    pub renderer: Renderer,
}

pub struct UiManagers {
    pub font_manager: FontManager,
}

/// Struct used to configure the UI system
#[derive(Debug, Copy, Clone)]
pub struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
    pub default_window_size: Size,
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
/// * `config` - Struct that modifies how the ui system is initialised
#[instrument]
pub fn init_ui_system(title: &str, config: UiConfig) -> eyre::Result<UiSystem> {
    let display;
    let mut imgui_context;
    let event_loop;
    let mut platform;
    let renderer;

    //TODO: More config options
    trace!("cloning title");
    let title = title.to_owned();
    trace!("creating [winit] event loop");
    event_loop = EventLoop::new();
    trace!("creating [glutin] context builder");
    let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
        .with_vsync(config.vsync)
        .with_hardware_acceleration(config.hardware_acceleration);
    trace!("creating [winit] window builder");
    let window_builder = WindowBuilder::new().with_title(title).with_inner_size(config.default_window_size).with_maximized(true);
    //TODO: Configure
    trace!("creating display");
    display = Display::new(window_builder, glutin_context_builder, &event_loop)
        .expect("could not initialise display");
    trace!("Creating [imgui] context");
    imgui_context = Context::create();
    imgui_context.set_ini_filename(PathBuf::from(log_expr_val!(IMGUI_SETTINGS_FILE_PATH)));
    imgui_context.set_log_filename(PathBuf::from(log_expr_val!(IMGUI_LOG_FILE_PATH)));
    trace!("enabling docking config flag");
    imgui_context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    trace!("creating font manager");
    let font_manager = FontManager::new()?;

    //TODO: High DPI setting
    trace!("creating [winit] platform");
    platform = WinitPlatform::init(&mut imgui_context);
    trace!("attaching window");
    platform.attach_window(
        imgui_context.io_mut(),
        display.gl_window().window(),
        HiDpiMode::Default,
    );
    trace!("creating [glium] renderer");
    renderer = Renderer::init(&mut imgui_context, &display).expect("failed to create renderer");

    trace!("done");

    match clipboard_integration::clipboard_init() {
        Ok(clipboard_backend) => {
            trace!("have clipboard support: {clipboard_backend:?}");
            imgui_context.set_clipboard_backend(clipboard_backend);
        }
        Err(error) => {
            warn!("could not initialise clipboard: {error}")
        }
    }

    Ok(UiSystem {
        backend: UiBackend {
            event_loop,
            display,
            imgui_context,
            platform,
            renderer,
        },
        managers: UiManagers {
            font_manager
        },
    })
}

impl UiManagers {
    pub fn render_ui_managers_window(&mut self, ui: &Ui) {
        ui.window("UI Management")
          .size([300.0, 110.0], Condition::FirstUseEver)
          .build(|| {
              self.font_manager.render_font_selector(ui);
          });
    }
}
