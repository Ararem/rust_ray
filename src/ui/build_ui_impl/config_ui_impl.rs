use crate::config::init_time::InitTimeAppConfig;
use crate::config::run_time::RuntimeAppConfig;
use crate::config::{load_config_from_disk, read_config_value};
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_display;
use crate::ui::build_ui_impl::shared::error_display::display_eyre_report;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::Report;
use imgui::Ui;
use tracing::{debug, trace, trace_span, warn};

pub(super) fn render_config_ui(ui: &Ui, visible: bool) -> FallibleFn {
    let span_render_config =
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_config").entered();
    if !visible {
        trace!(target: UI_TRACE_BUILD_INTERFACE, "not visible");
        return Ok(());
    }

    unsafe {
        trace!(
            target: UI_TRACE_BUILD_INTERFACE,
            "[Button] Reload From Disk"
        );

        // I need this to be shared across frames somehow, so i have to make it static mut :(
        // The way this is written is weird and I don't like it, but I'm constrained by ImGUI and how it wants it's functions called
        static mut LOAD_CONFIG_ERROR: Option<Report> = None;
        const MODAL_NAME: &str = "Could not reload from disk";

        if ui.button("Reload From Disk") {
            debug!(
                target: UI_DEBUG_USER_INTERACTION,
                "[Button] Reload From Disk pressed"
            );
            // Try loading, and if there was an error, log it and open a popup modal for the user
            if let Err(report) = load_config_from_disk() {
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    report = format_report_display(&report),
                    "could not load config from disk"
                );
                ui.open_popup(MODAL_NAME);
                LOAD_CONFIG_ERROR = Some(report);
            }
        }

        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "config_error_modal").in_scope(||{
        // If we have an error, its modal time....
        // Also demonstrate passing a bool, this will create a regular close button which
        // will close the popup. Note that the visibility state of popups is owned by imgui, so the input value
        // of the bool actually doesn't matter here.
        let mut opened_sesame = true;
        let maybe_token = ui.modal_popup_config(MODAL_NAME).opened(&mut opened_sesame).begin_popup();
        match maybe_token {
            None => {
                // Modal closed, clear the current error
                LOAD_CONFIG_ERROR = None;
                trace!(
                    target: UI_TRACE_BUILD_INTERFACE,
                    "modal not visible"
                );
            }
            Some(token) => {
                if let Some(ref report) = LOAD_CONFIG_ERROR {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "displaying error");
                    display_eyre_report(ui, report);
                } else {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "don't have a config error!?!?");
                    warn!(target: GENERAL_WARNING_NON_FATAL, "config error modal was opened but we don't have an error to display. this probably shouldn't have happened");
                    ui.text_colored(
                        read_config_value(|config| config.runtime.ui.colours.severity.warning),
                        "This popup shouldn't be visible, sorry about that. Normally it would show you an error that happened with reloading the config, but we don't have any error to display (yay)",
                        );
                    ui.close_current_popup();
                }

                trace!(target: UI_TRACE_BUILD_INTERFACE, "Close button");
                ui.spacing();
                if ui.button("Close") {
                    debug!(target: UI_DEBUG_USER_INTERACTION, "[Button] Pressed Close config error modal");
                    ui.close_current_popup();
                }
                token.end();
            }
        };});
    }

    span_render_config.exit();
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
