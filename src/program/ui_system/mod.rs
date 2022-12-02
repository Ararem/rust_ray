use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use color_eyre::{eyre, Help, IndentedSection, Report};
use color_eyre::owo_colors::OwoColorize;
use glium::{Display, glutin};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use imgui::{Condition, Context, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_winit_support::winit::dpi::Size;
use lazy_static::__Deref;
use lazy_static::lazy_static;
use shadow_rs::new;
use structx::*;
use tracing::{error, info, instrument, trace, warn};

use nameof::*;

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
    pub fn render_ui_managers_window(&mut self, ui: &Ui, opened: &mut bool) {
        if !*opened { return; }
        ui.window("UI Management")
          .opened(opened)
          .size([300.0, 110.0], Condition::FirstUseEver)
          .build(|| {
              self.font_manager.render_font_selector(ui);

              UiManagers::render_framerate_graph(ui);
          });
    }

    fn render_framerate_graph(ui: &Ui) {
        static mut i: i32 = 0;
        lazy_static! {
            static ref FRAME_TIMES: Mutex<FrameTimes> = Mutex::new(FrameTimes{deltas:Vec::new(), fps:Vec::new()});
        }
        static NUM_FRAME_TIMES_TO_TRACK: usize = 120usize;
        #[derive(Debug, Clone)]
        struct FrameTimes {
            /// ΔT values, in milliseconds
            ///
            /// # See Also
            /// * [delta_time](imgui::Io::delta_time) - Where this value is obtained from
            deltas: Vec<f32>,
            /// Frames per second
            ///
            /// Inverse of [deltas](FrameTimes::deltas)
            fps: Vec<f32>
        }
        let mut guard_frame_times = match FRAME_TIMES.lock() {
            Err(poisoned) => {
                let report = Report::msg(format!("{} mutex was poisoned", name_of!(FRAME_TIMES)))
                    .note("Perhaps [render_ui_managers()] was called multiple times (async), and one call failed, causing the FRAME_TIME mutex to be poisoned by that failure?\nNote: This **should never happen** as the UI should be single-threaded");
                error!("{}", report);
                poisoned.into_inner()
            }
            Ok(guard) => guard,
        };


        //TODO: See if there's a faster way
        let frame_times = guard_frame_times.deref_mut();
        unsafe {
            i += 1;
            if i % 1000 == 1 {
                let delta = ui.io().delta_time;
                frame_times.deltas.insert(0, delta * 1000.0);
                frame_times.fps.insert(0, 1f32 / delta);
                frame_times.deltas.truncate(NUM_FRAME_TIMES_TO_TRACK);
                frame_times.fps.truncate(NUM_FRAME_TIMES_TO_TRACK);
            }
        }
        let x: VecDeque<f32> = VecDeque::new();
        x.fro
        ui.plot_lines("Off Frame Times (ms)", &frame_times.deltas)
          .values_offset(frame_times.deltas.len() / 2)
          .build();
        ui.plot_lines("Frame Times (ms)", &frame_times.deltas)
          .build();
        ui.plot_lines("Frame Rates", &frame_times.fps)
          .build();
    }
}
