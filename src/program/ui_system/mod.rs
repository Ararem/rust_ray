mod clipboard_integration;
mod font_manager;

use crate::log_expr;
use color_eyre::eyre;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::{glutin, Display};
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use tracing::{debug, debug_span, instrument, trace, warn};

/*
TODO:   Add support for different renderers (glow, glium, maybe d3d12, dx11, wgpu) and backend platforms (winit, sdl2)
        Should probably add an enum selection to [UiConfig]. Also see if it's possible to change hotly or requires a reload
*/

/// Struct that encapsulates the UI system components
pub struct UiSystem {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub imgui_context: Context,
    pub platform: WinitPlatform,
    /// The renderer that renders the current UI system
    pub renderer: Renderer,
}

/// Struct used to configure the UI system
#[derive(Debug, Copy, Clone)]
pub struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
/// * `config` - Struct that modifies how the ui system is initialised
#[instrument]
pub fn init_ui_system(title: &str, config: UiConfig) -> eyre::Result<UiSystem> {
    let display;
    let mut imgui;
    let event_loop;
    let mut platform;
    let renderer;

    {
        let _span = debug_span!("window_internals").entered();
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
        let window_builder = WindowBuilder::new().with_title(title); //TODO: Configure
        trace!("creating display");
        display = Display::new(window_builder, glutin_context_builder, &event_loop)
            .expect("could not initialise display");
        trace!("Creating [imgui] context");
        imgui = Context::create();
        // imgui.set_ini_filename(Some(PathBuf::from("./imgui.ini")));
        // imgui.set_log_filename()

        font_manager::add_fonts(&mut imgui);

        //TODO: High DPI setting
        trace!("creating [winit] platform");
        platform = WinitPlatform::init(&mut imgui);
        trace!("attaching window");
        platform.attach_window(
            imgui.io_mut(),
            display.gl_window().window(),
            HiDpiMode::Default,
        );
        trace!("creating [glium] renderer");
        renderer = Renderer::init(&mut imgui, &display).expect("failed to create renderer");

        trace!("done")
    }

    match clipboard_integration::clipboard_init() {
        Ok(clipboard_backend) => {
            trace!("have clipboard support: {clipboard_backend:?}");
            imgui.set_clipboard_backend(clipboard_backend);
        }
        Err(error) => {
            warn!("could not initialise clipboard: {error}")
        }
    }


    Ok(UiSystem {
        event_loop,
        display,
        imgui_context: imgui,
        platform,
        renderer,
    })
}
