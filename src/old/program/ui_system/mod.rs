use std::cmp::min;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Mutex;

use color_eyre::{eyre, Help, Report};
use glium::{Display, glutin};
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use imgui::{Condition, Context, ItemHoveredFlags, SliderFlags, TreeNodeFlags, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_winit_support::winit::dpi::Size;
use lazy_static::lazy_static;
use nameof::*;
use tracing::{error, instrument, trace, warn};

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
        lazy_static! {
            static ref FRAME_TIMES: Mutex<FrameTimes> = Mutex::new(FrameTimes{deltas:Vec::new(), fps:Vec::new()});
        }
        /// The number of frames to be recorded into the buffer at most
        static mut NUM_FRAMES_TO_TRACK: usize = 3600usize;
        /// The number of frames to display in the UI
        static mut NUM_FRAMES_TO_DISPLAY: usize = 3600usize;
        /// Arbitrary hard limit on how many frames we are allowed to track.
        const HARD_LIMIT_MAX_FRAMES_TO_TRACK: u64 = 64_000u64;
        #[derive(Debug, Clone)]
        /// Struct that stores arrays of floats for frame times (ΔT) and frame-rates
        ///
        ///
        /// # Performance Notes
        /// Although using a [Vec] as a FIFO queue normally would be a bad idea, since inserting at `[0]` always causes the entire vec to be shifted
        /// In benchmarks, it was actually *much* faster that using any other collection types:
        /// * [VecDeque] - Wouldn't work because in order to plot, a slice `[f32]` needs to be passed, and this is very tricky to get from a [VecDeque]
        /// * [SliceDeque] - Worked almost identically to [Vec], but was orders of magnitudes slower (`~1 us` for [SliceDeque] vs `~22ns` for [Vec], at 120 frames stored).
        ///     At extreme frame counts (`~12000` frames), it did gain a slight advantage (`1us` vs `1.4us`), indicating that [SliceDeque] has `O(1)` performance, but has a massive overhead comparatively to [Vec]
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

        unsafe { // Has to be unsafe because of mut static variables
            if !(ui.collapsing_header("Frame Timings", TreeNodeFlags::empty())) {
                return;
            }

            let mut guard_frame_times = match FRAME_TIMES.lock() {
                Err(poisoned) => {
                    let report = Report::msg(format!("{} mutex was poisoned", name_of!(FRAME_TIMES)))
                        .note("Perhaps [render_ui_managers()] was called multiple times (async), and one call failed, causing the FRAME_TIME mutex to be poisoned by that failure?\nNote: This **should never happen** as the UI should be single-threaded");
                    log_err
                    poisoned.into_inner()
                }
                Ok(guard) => guard,
            };

            let frame_times = guard_frame_times.deref_mut();
            let delta = ui.io().delta_time;
            // We insert into the front (start) of the Vec, then truncate the end, ensuring that the values get pushed along and we don't go over our limit
            frame_times.deltas.insert(0, delta * 1000.0);
            frame_times.fps.insert(0, 1f32 / delta);
            frame_times.deltas.truncate(NUM_FRAMES_TO_TRACK);
            frame_times.fps.truncate(NUM_FRAMES_TO_TRACK);

            // Note the [0..min(NUM_FRAMES_TO_DISPLAY, frame_times.XXX.len())-1]
            // This ensures that we don't try to take a slice that's bigger than the amount we have in the Vec
            // Don't have to worry about the `-1` if `len() == 0`, since len() should never `== 0`: we always have at least 1 frame since we insert above, and NUM_FRAMES_TO_DISPLAY should always be >=1
            let num_delta_frames = min(NUM_FRAMES_TO_DISPLAY, frame_times.deltas.len());
            ui.plot_lines("Frame Times (ms)", &frame_times.deltas[0..num_delta_frames - 1])
              .overlay_text(format!("{num_delta_frames} frames"))
              .build();
            let num_fps_frames = min(NUM_FRAMES_TO_DISPLAY, frame_times.fps.len());
            ui.plot_lines("Frame Rates", &frame_times.fps[0..num_fps_frames - 1])
              .overlay_text(format!("{num_fps_frames} frames"))
              .build();

            let mut num_track_frames_u64: u64 = NUM_FRAMES_TO_TRACK as u64; // Might fail on 128-bit systems, but eh
            ui.slider_config("Num Tracked Frames", 1, HARD_LIMIT_MAX_FRAMES_TO_TRACK)
              .flags(SliderFlags::LOGARITHMIC)
              .build(&mut num_track_frames_u64);
            NUM_FRAMES_TO_TRACK = num_track_frames_u64 as usize;
            if ui.is_item_hovered() {
                ui.tooltip_text("The maximum amount of frames that can be stored at one time. You probably want to leave this alone and modify [Num Displayed Frames] instead");
            }

            let mut num_displayed_frames_u64: u64 = NUM_FRAMES_TO_DISPLAY as u64; // Might fail on 128-bit systems, but eh
            num_displayed_frames_u64 = min(num_displayed_frames_u64, num_track_frames_u64); // Don't allow it to go over num_track_frames
            ui.slider_config("Num Displayed Frames", 1, min(HARD_LIMIT_MAX_FRAMES_TO_TRACK, num_track_frames_u64))
              .flags(SliderFlags::LOGARITHMIC)
              .build(&mut num_displayed_frames_u64);
            NUM_FRAMES_TO_DISPLAY = num_displayed_frames_u64 as usize;
            if ui.is_item_hovered() {
                ui.tooltip_text("The number of frames that will be displayed in the plot. Must be <= [Num Tracked Frames]. Will also be automatically limited if there are not enough frames stored to be displayed (until there are enough)");
            }
        }
    }
}
