use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct FrameInfoConfig {
    /// Size of the 'chunks' used when averaging frame values
    pub chunked_average_smoothing_size: usize,
    /// Toggle for if the minimum value shown should always be zero
    pub min_always_at_zero: bool,
    /// The number of frames of information to display
    pub num_frames_to_display: usize,
    /// The number of frames of information to record. This value will be clamped to a max of [MAX_FRAMES_TO_TRACK][super::super::config::compile_time::ui_config::MAX_FRAMES_TO_TRACK]
    pub num_frames_to_track: usize,
    /// Value that controls how fast the range for the frame info values is lerped. lower values make a smoother (slower) lerp
    pub smooth_speed: f32,
}

impl Default for FrameInfoConfig {
    fn default() -> Self {
        Self {
            chunked_average_smoothing_size: 8,
            min_always_at_zero: true,
            num_frames_to_track: 32_000,
            num_frames_to_display: 1920,
            smooth_speed: 0.03,
        }
    }
}
