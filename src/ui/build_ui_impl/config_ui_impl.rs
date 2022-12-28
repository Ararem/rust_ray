use crate::config::init_time::InitTimeAppConfig;
use crate::config::read_config_value;
use crate::config::run_time::RuntimeAppConfig;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use imgui::{Ui};

pub(super) fn render_config_ui(ui: &Ui, visible: bool) -> FallibleFn {
    if !visible {
        return Ok(());
    }

    let colours = read_config_value(|config| config.runtime.ui.colours);
    ui.text_colored(colours.error, "error");
    ui.text_colored(colours.warning, "warning");
    ui.text_colored(colours.good, "good");
    ui.text_colored(colours.severe_error, "severe_error");

    Ok(())
}

impl UiItem for InitTimeAppConfig {
    fn render(&mut self, _ui: &Ui, _visible: bool) -> FallibleFn {
        Ok(())
    }
}
impl UiItem for RuntimeAppConfig {
    fn render(&mut self, _ui: &Ui, _visible: bool) -> FallibleFn {
        Ok(())
    }
}

/*


       // ===== Sliders for control =====


       // usize can't be used in a slider, so we have to cast to u64, use that, then case back
       type SliderType = u64; // Might fail on 128-bit systems where usize > u64, but eh
       let mut num_track_frames_compat: SliderType = *track_frames as SliderType;
       ui.slider_config("Num Tracked Frames", 1, MAX_FRAMES_TO_TRACK as SliderType)
           .display_format(format!("%u ({capped})", capped = num_frame_infos).as_str()) // Also display how many frames we are actually tracking currently.
           .flags(SliderFlags::LOGARITHMIC)
           .build(&mut num_track_frames_compat);
       *track_frames = num_track_frames_compat as usize;
       if ui.is_item_hovered() {
           ui.tooltip_text("The maximum amount of frames that can be stored at one time. You probably want to leave this alone and modify [Num Displayed Frames] instead");
       }

       *displayed_frames = min(*displayed_frames, *track_frames); // Don't allow it to go over num_track_frames
       let mut num_displayed_frames_compat: SliderType = *displayed_frames as SliderType; // Might fail on 128-bit systems, but eh
       ui.slider_config(
           "Num Displayed Frames",
           1,
           min(MAX_FRAMES_TO_TRACK, *track_frames) as SliderType,
       )
       .display_format(format!("%u ({capped})", capped = info_range_end + 1).as_str()) // Also display how many frames we are actually displaying (in case there aren't enough to show)
       .flags(SliderFlags::LOGARITHMIC)
       .build(&mut num_displayed_frames_compat);
       *displayed_frames = num_displayed_frames_compat as usize;
       if ui.is_item_hovered() {
           ui.tooltip_text("The number of frames that will be displayed in the plot. Must be <= [Num Tracked Frames]. Will also be automatically limited if there are not enough frames stored to be displayed (until there are enough)");
       }

       let mut smoothing_compat: SliderType = self.scale_smoothing as SliderType; // Might fail on 128-bit systems, but eh
       ui.slider_config("Plot Range Smoothing", 1, 256 as SliderType)
           .flags(SliderFlags::LOGARITHMIC)
           .build(&mut smoothing_compat);
       self.scale_smoothing = smoothing_compat as usize;
       if ui.is_item_hovered() {
           ui.tooltip_text("Amount of smoothing to apply when calculating the range values for plotting. Higher values increase smoothing, de-focusing peaks and spikes");
       }

*/
