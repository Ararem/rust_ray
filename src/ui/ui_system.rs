//! Module that contains the structs used in the [crate::ui] module
use crate::ui::font_manager::FontManager;
use glium::glutin::event_loop::EventLoop;
use glium::Display;
use imgui::Context;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::WinitPlatform;

//TODO: Debug impls for these UI structs
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

#[derive(Debug, Clone)]
pub(in crate::ui) struct UiManagers {
    pub font_manager: FontManager,
    pub frame_info: FrameInfo,
}

/// Struct that stores arrays of floats for frame times (ΔT) and frame-rates
///
///
/// # Performance Notes
/// Although using a [Vec] as a FIFO queue normally would be a bad idea, since inserting at `[0]` always causes the entire vec to be shifted
/// In benchmarks, it was actually *much* faster that using any other collection types:
/// * [VecDeque] - Wouldn't work because in order to plot, a slice `[f32]` needs to be passed, and this is very tricky to get from a [VecDeque]
/// * [SliceDeque] - Worked almost identically to [Vec], but was orders of magnitudes slower (`~1 us` for [SliceDeque] vs `~22ns` for [Vec], at 120 frames stored).
/// At extreme frame counts (`~12000` frames), it did gain a slight advantage (`1us` vs `1.4us`),
/// indicating that [SliceDeque] has `O(1)` performance, but has a massive overhead comparatively to [Vec]
#[derive(Debug, Clone)]
pub(in crate::ui) struct FrameInfo {
    /// ΔT values, in milliseconds
    ///
    /// # See Also
    /// * [delta_time](imgui::Io::delta_time) - Where this value is obtained from
    pub deltas: Vec<f32>,
    /// Frames per second
    ///
    /// Inverse of [deltas](FrameTimes::deltas)
    pub fps: Vec<f32>,

    // Moving averages for displaying the above vecs
    pub smooth_delta_min: f32,
    pub smooth_delta_max: f32,
    pub smooth_fps_min: f32,
    pub smooth_fps_max: f32,
}

impl FrameInfo {
    pub fn new() -> Self {
        Self {
            deltas: vec![],
            smooth_delta_min: 0.0,
            smooth_delta_max: 0.0,
            smooth_fps_min: 0.0,
            smooth_fps_max: 0.0,
            fps: vec![],
        }
    }
}
