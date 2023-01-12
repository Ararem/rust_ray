use crate::config::compile_time::ui_config::MAX_FRAMES_TO_TRACK;
use crate::config::init_time::InitTimeAppConfig;
use crate::config::run_time::ui_config::theme::Colour;
use crate::config::run_time::RuntimeAppConfig;
use crate::config::{load_config_from_disk, read_config_value, save_config_to_disk, update_config};
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_display;
use crate::ui::build_ui_impl::shared::error_display::an_error_occurred;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use backtrace::trace;
use color_eyre::Report;
use criterion::AxisScale::Logarithmic;
use imgui::{ColorEditFlags, ColorFormat, ColorPreview, SliderFlags, TreeNodeFlags, Ui};
use indoc::{formatdoc, indoc};
use tracing::subscriber::with_default;
use tracing::{debug, trace, trace_span, warn};
use vek::num_traits::real::Real;

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

    update_config(|cfg| {
        //Do a check to make sure we aren't overwriting any other external changes
        if cfg != &original_config {
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "original and current config didn't match: something modified config externally while config UI was being rendered"
            );
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
        let init_config_node = match ui.tree_node("Init Config") {
            None => {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "init config collapsed");
                return Ok(());
            }
            Some(node) => node,
        };

        if let Some(ui_config_node) = ui.tree_node("UI") {
            // With longer labels, the labels don't fit on the screen unless we give them a bit more width
            let width_token = ui.push_item_width(ui.content_region_avail()[0] * 0.5);
            let cfg = &mut self.ui_config;
            if ui.checkbox("VSync", &mut cfg.vsync) {
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed vsync => {}", cfg.vsync);
            }
            if ui.checkbox("Start Maximised", &mut cfg.start_maximised) {
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed start_maximised => {}", cfg.start_maximised);
            }
            // Since we only have 3 possible values here, I find it acceptable to use hardcoded values
            // This does mean that everything has to match perfectly, or bugs will happen
            const HARDWARE_ACCELERATION_OPTIONS: [&'static str; 3] = ["Automatic", "Enabled", "Disabled"];
            let mut hw_accel_idx = match cfg.hardware_acceleration {
                None => 0,
                Some(true) => 1,
                Some(false) => 2,
            };
            if ui.combo_simple_string("Hardware acceleration", &mut hw_accel_idx, &HARDWARE_ACCELERATION_OPTIONS) {
                let accel = match hw_accel_idx {
                    0 => None,
                    1 => Some(true),
                    2 => Some(false),
                    bad_value => unreachable!("There are only 3 option for hardware acceleration, but the value was out of range: {}", bad_value),
                };
                cfg.hardware_acceleration = accel;
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed hardware acceleration => {:?}", cfg.hardware_acceleration);
            }
            // Multisampling must be a power of 2, so fake it by showing the exponent
            let mut multisampling_exponent: u16 = (cfg.multisampling as f32).log2() as u16;
            if ui
                .slider_config("Multisampling", 0, 4)
                .display_format(format!("{}", 1u16 << multisampling_exponent))
                .build(&mut multisampling_exponent)
            {
                cfg.multisampling = 1u16 << multisampling_exponent;
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed multisampling => {}", cfg.multisampling);
            }

            width_token.end();
            ui_config_node.end();
        } else {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "ui config collapsed")
        }

        init_config_node.end();
        span_render.exit();
        Ok(())
    }
}
impl UiItem for RuntimeAppConfig {
    fn render(&mut self, ui: &Ui, _visible: bool) -> FallibleFn {
        let span_render = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_runtime_config", runtime_config=?self).entered();
        trace!(target: UI_TRACE_BUILD_INTERFACE, "runtime config collapsing header");
        let init_config_node = match ui.tree_node("Runtime Config") {
            None => {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "runtime config collapsed");
                return Ok(());
            }
            Some(node) => node,
        };

        if let Some(ui_config_node) = ui.tree_node("UI") {
            // With longer labels, the labels don't fit on the screen unless we give them a bit more width
            let width_token = ui.push_item_width(ui.content_region_avail()[0] * 0.5);
            let ui_cfg = &mut self.ui;

            if ui.slider("Font Oversampling", 1, 4, &mut ui_cfg.font_oversampling) {
                trace!(target: UI_DEBUG_USER_INTERACTION, "changed font_oversampling => {}", ui_cfg.font_oversampling);
            }

            if let Some(frame_info_node) = ui.tree_node("Frame Info") {
                // With longer labels, the labels don't fit on the screen unless we give them a bit more width
                let width_token = ui.push_item_width(ui.content_region_avail()[0] * 0.5);
                let frame_cfg = &mut ui_cfg.frame_info;

                if ui.checkbox("Always show 0", &mut frame_cfg.min_always_at_zero) {
                    trace!(target: UI_DEBUG_USER_INTERACTION, "changed min_always_at_zero => {}", frame_cfg.min_always_at_zero);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("When displaying frame rate and frame time graphs, whether to always have the bottom of the graph be at 0 (rather than the approximate smallest value)");
                }

                if slider_usize(ui, &mut frame_cfg.num_frames_to_track, SliderFlags::LOGARITHMIC, 69, MAX_FRAMES_TO_TRACK, "Max Tracked Frames", None) {
                    trace!(target: UI_DEBUG_USER_INTERACTION, "changed num_frames_to_track => {}", frame_cfg.num_frames_to_track);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(indoc! {r"
                    The maximum amount of frames that can be stored at one time.\
                    You probably want to leave this alone and modify [Num Displayed Frames] instead
                    "});
                }

                if slider_usize(
                    ui,
                    &mut frame_cfg.num_frames_to_display,
                    SliderFlags::LOGARITHMIC,
                    1,
                    frame_cfg.num_frames_to_track,
                    "Num Displayed Frames",
                    None,
                ) {
                    trace!(target: UI_DEBUG_USER_INTERACTION, "changed num_frames_to_display => {}", frame_cfg.num_frames_to_display);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(indoc! {r"
                    The maximum amount of frames that will be displayed in the frame info interface.
                    Cannot be set higher than [Max Tracked Frames], and will be soft-limited if there are insufficient frames to display
                    (i.e. if only X frames are stored, only X will be shown, until X is at least this value)
                    "});
                }

                if slider_usize(ui, &mut frame_cfg.chunked_average_smoothing_size, SliderFlags::LOGARITHMIC, 1, 256, "Frame Smoothing Interval", None) {
                    trace!(
                        target: UI_DEBUG_USER_INTERACTION,
                        "changed chunked_average_smoothing_size => {}",
                        frame_cfg.chunked_average_smoothing_size
                    );
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(indoc! {r"
                    When calculating the value range for plotting, the chunk size in which to average values.
                    Higher values increase average more values, smoothing the min/max calculation (by reducing outliers), and de-focusing peaks and spikes
                    "});
                }

                if ui.slider_config("Lerp speed", 0.00001, 0.1).flags(SliderFlags::LOGARITHMIC).build(&mut frame_cfg.smooth_speed) {
                    trace!(target: UI_DEBUG_USER_INTERACTION, "changed smooth_speed => {}", frame_cfg.smooth_speed);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(indoc! {r#"
                    The amount by which to lerp between old values and new values, each frame. Smaller values will result in a smaller interpolation per-frame,
                    Which will "slow down" the effect and result in more gradual changes
                    "#});
                }

                width_token.end();
                frame_info_node.end();
            } else {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "frame info config collapsed")
            }

            if let Some(colours_node) = ui.tree_node("Colours") {
                let col_cfg = &mut ui_cfg.colours;
                macro_rules! colour {
                    ($name:expr, $field:expr) => {
                        colour(ui, &mut $field, $name);
                    };
                }
                fn colour(ui: &Ui, field: &mut Colour, name: &str) {
                    //TODO: Should this be another part of the config?
                    let picker = ui
                        .color_picker4_config(name, field)
                        .format(ColorFormat::Float)
                        .tooltip(true)
                        .alpha(true)
                        .alpha_bar(true)
                        .display_hex(true)
                        .display_hsv(true)
                        .display_rgb(true)
                        .side_preview(true)
                        .small_preview(false)
                        .preview(ColorPreview::HalfAlpha)
                        .options(true);
                    if picker.build() {
                        trace!(target: UI_DEBUG_USER_INTERACTION, "changed ui colour {name} => {field:?}");
                    }
                }

                if ui.collapsing_header("Text Colours", TreeNodeFlags::empty()) {
                    colour!("Normal", col_cfg.text.normal);
                    colour!("Subtle", col_cfg.text.subtle);
                    colour!("Accent", col_cfg.text.accent);
                } else {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "text colours header collapsed");
                }
                if ui.collapsing_header("Severity Colours", TreeNodeFlags::empty()) {
                    colour!("Good", col_cfg.severity.good);
                    colour!("Neutral", col_cfg.severity.neutral);
                    colour!("Note", col_cfg.severity.note);
                    colour!("Warning", col_cfg.severity.warning);
                    colour!("Very Bad", col_cfg.severity.very_bad);
                } else {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "severity colours header collapsed");
                }
                if ui.collapsing_header("Value Colours", TreeNodeFlags::empty()) {
                    colour!("Error Event", col_cfg.value.level_error);
                    colour!("Warn Event", col_cfg.value.level_warn);
                    colour!("Info Event", col_cfg.value.level_info);
                    colour!("Debug Event", col_cfg.value.level_debug);
                    colour!("Trace Event", col_cfg.value.level_trace);
                    ui.separator();
                    colour!("Tracing Event Name", col_cfg.value.tracing_event_name);
                    colour!("Tracing Field Name", col_cfg.value.tracing_event_field_name);
                    colour!("Tracing Field Value", col_cfg.value.tracing_event_field_value);
                    ui.separator();
                    colour!("Function", col_cfg.value.function_name);
                    colour!("File Path", col_cfg.value.file_location);
                    ui.separator();
                    colour!("Error Message", col_cfg.value.error_message);
                    ui.separator();
                    colour!("Value Label", col_cfg.value.value_label);
                    ui.separator();
                    colour!("Misc Value", col_cfg.value.misc_value);
                    colour!("Missing Value", col_cfg.value.missing_value);
                    colour!("Symbols", col_cfg.value.symbol);
                    colour!("Numbers", col_cfg.value.number);
                } else {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "value colours header collapsed");
                }
            }
            width_token.end();
            ui_config_node.end();
        } else {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "ui config collapsed")
        }

        init_config_node.end();
        span_render.exit();
        Ok(())
    }
}

fn slider_usize(ui: &Ui, val: &mut usize, flags: SliderFlags, min: usize, max: usize, label: &str, display_format: Option<&str>) -> bool {
    let mut compat_u64 = *val as u64;
    let mut slider = ui.slider_config(label, min as u64, max as u64).flags(flags);
    if let Some(fmt) = display_format {
        slider = slider.display_format(fmt);
    }
    let changed = slider.build(&mut compat_u64);
    *val = compat_u64 as usize;
    changed
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
