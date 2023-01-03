/// The minimum size (in pixels) that can be used when selecting the size of a font
pub const MIN_FONT_SIZE: f32 = 8f32;
/// The maximum allowed size for a font (in pixels)
pub const MAX_FONT_SIZE: f32 = 128f32;
/// The maximum number of frames (see [crate::ui::ui_system::FrameInfo]) that should be tracked
pub const MAX_FRAMES_TO_TRACK: usize = 64_000;

//TODO: Get rid of these, make them constraints in the IMGUI code to display the config
