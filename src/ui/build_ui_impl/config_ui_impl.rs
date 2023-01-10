use backtrace::trace;
use crate::config::init_time::InitTimeAppConfig;
use crate::config::{load_config_from_disk, read_config_value, save_config_to_disk, update_config};
use crate::config::run_time::RuntimeAppConfig;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_display;
use crate::ui::build_ui_impl::shared::error_display::an_error_occurred;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::Report;
use imgui::{TreeNodeFlags, Ui};
use tracing::{debug, trace, trace_span, warn};

pub(super) fn render_config_ui(ui: &Ui, visible: bool) -> FallibleFn {
    let span_render_config = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_config").entered();
    if !visible {
        trace!(target: UI_TRACE_BUILD_INTERFACE, "not visible");
        return Ok(());
    }

    trace!(target: UI_TRACE_BUILD_INTERFACE, "[Button] Reload from Disk");
    if ui.button("Reload From Disk") {
        debug!(target: UI_DEBUG_USER_INTERACTION, "[Button] Reload From Disk pressed");
        // Try loading, and if there was an error, log it and open a popup modal for the user
        if let Err(report) = load_config_from_disk() {
            warn!(target: GENERAL_WARNING_NON_FATAL, report = format_report_display(&report), "could not load config from disk");
            an_error_occurred(report);
            an_error_occurred(Report::msg("Test"))
        }
    }

    trace!(target: UI_TRACE_BUILD_INTERFACE, "[Button] Save to Disk");
    if ui.button("Save to Disk") {
        debug!(target: UI_DEBUG_USER_INTERACTION, "[Button] Save to Disk pressed");
        // Try saving, and if there was an error, log it and open a popup modal for the user
        if let Err(report) = save_config_to_disk() {
            warn!(target: GENERAL_WARNING_NON_FATAL, report = format_report_display(&report), "could not save config to disk");
            an_error_occurred(report);
            an_error_occurred(Report::msg("Test"))
        }
    }

    // This is a little iffy because we're cloning the config, then setting it later
    // There is a chance that something will modify the config while we are modifying the copy,
    // and then that change will be overwritten later.
    // We can't (easily) work around this, but we can detect it by cloning the original,
    // and then checking the original against the current when we go to update it
    // If they differ, something was changed and we are overwriting it
    let original_config = read_config_value(|config| config.clone());
    let mut modified_config = original_config.clone();

    modified_config.init.render(ui, true)?;
    modified_config.runtime.render(ui, true)?;


    update_config(|cfg|{
        //Do a check to make sure we aren't overwriting any other external changes
        if cfg != &original_config{
            warn!(target: GENERAL_WARNING_NON_FATAL, "original and current config didn't match: something modified config externally while config UI was being rendered");
        }
        *cfg = modified_config;
    });


    span_render_config.exit();
    Ok(())
}

impl UiItem for InitTimeAppConfig {
    fn render(&mut self, ui: &Ui, _visible: bool) -> FallibleFn {
        let span_render = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_init_config", init_config=?self).entered();
        trace!(target: UI_TRACE_BUILD_INTERFACE, "init config collapsing header");
        if !ui.collapsing_header("Init Config", TreeNodeFlags::empty()){
            trace!(target: UI_TRACE_BUILD_INTERFACE, "init config collapsed");
            return Ok(());
        }
        if ui.collapsing_header("UI", TreeNodeFlags::empty()){
            let cfg = &mut self.ui_config;
            if ui.checkbox("VSync", &mut cfg.vsync){
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed vsync => {}", cfg.vsync);
            }
            if ui.checkbox("Start Maximised", &mut cfg.start_maximised){
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed start_maximised => {}", cfg.start_maximised);
            }
            // Since we only have 3 possible values here, I find it acceptable to use hardcoded values
            // This does mean that everything has to match perfectly, or bugs will happen
            const HARDWARE_ACCELERATION_OPTIONS: [&'static str;3] = ["Automatic", "Enabled", "Disabled"];
            let mut hw_accel_idx = match cfg.hardware_acceleration{
                None => 0,
                Some(true) => 1,
                Some(false) => 2
            };
            if ui.list_box("Hardware acceleration", &mut hw_accel_idx, &HARDWARE_ACCELERATION_OPTIONS, 3){
                let accel = match hw_accel_idx{
                    0 => None,
                    1 => Some(true),
                    2 => Some(false),
                    bad_value => unreachable!("There are only 3 option for hardware acceleration, but the value was out of range: {bad_value}")
                };
                cfg.hardware_acceleration = accel;
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed hardware acceleration => {:?}", cfg.hardware_acceleration);
            }
            ui.label_text("Default window size", /*TODO: Impl*/ "TODO");
            ui.label_text("Multisampling", /*TODO: Impl*/ "TODO");
        }
        else{
            trace!(target: UI_TRACE_BUILD_INTERFACE, "ui config collapsed")
        }

        span_render.exit();
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
           ui.tooltip_text("The number of frames that will be displayed in the plot. Must be <= [Num Tracked Frames]. \
           Will also be automatically limited if there are not enough frames stored to be displayed (until there are enough)");
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
