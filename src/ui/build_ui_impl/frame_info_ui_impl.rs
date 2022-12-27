use crate::config::compile_time::ui_config::MAX_FRAMES_TO_TRACK;
use crate::config::Config;
use crate::helper::logging::event_targets::*;
use crate::ui::build_ui_impl::UiItem;
use crate::ui::ui_system::FrameInfo;
use crate::FallibleFn;
use imgui::{SliderFlags, TreeNodeFlags, Ui};
use itertools::*;
use std::cmp::{min};
use tracing::field::Empty;
use tracing::{trace, trace_span, warn};

impl UiItem for FrameInfo {
    fn render(&mut self, ui: &Ui, config: Config) -> FallibleFn {
        let span_render_framerate_graph =
            trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_framerate_graph").entered();

        let displayed_frames = &mut self.num_frames_to_display;
        let track_frames = &mut self.num_frames_to_track;
        let deltas = &mut self.frame_times.deltas;
        let fps = &mut self.frame_times.fps;

        // by placing this span before the header, we ensure that this always runs even when the header is collapsed
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "update_frames").in_scope(|| {
            let delta = ui.io().delta_time;
            // We insert into the front (start) of the Vec, then truncate the end, ensuring that the values get pushed along and we don't go over our limit
            deltas.insert(0, delta * 1000.0);
            fps.insert(0, 1f32 / delta);
            deltas.truncate(*track_frames);
            fps.truncate(*track_frames);
        });

        if !(ui.collapsing_header("Frame Timings", TreeNodeFlags::empty())) {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "frame timings collapsed");
            return Ok(());
        }

        // ===== Sliders for control =====

        // ensures that we don't try to take a slice that's bigger than the amount we have in the Vec
        // Don't have to worry about the `-1` if `len() == 0`, since len() should never `== 0`: we always have at least 1 frame since we insert above, and NUM_FRAMES_TO_DISPLAY should always be >=1
        let num_frame_infos = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "calc_num_frames").in_scope(||{
            let (len_d, len_f) = (deltas.len(), fps.len());
            let len;
            // We should always have the same number in both, but just to be safe, use the smaller one if they aren't the same
            if len_d != len_f{
                len = min(len_d, len_f);
                warn!(target: GENERAL_WARNING_NON_FATAL, "did not have same number of delta and fps frame infos: (delta: {len_d}, fps: {len_f}). should be same. using {len}");
            }
            else{
                len = len_d;
            }
            len
        });
        let info_range_end = min(*displayed_frames, num_frame_infos) - 1;

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

        //// ===== Plots =====

        fn chunked_smooth_minmax(vec: &[f32], chunk_size: usize) -> (f32, f32) {
            vec.iter()
                .chunks(chunk_size) // Group by 8 frames at a time
                .into_iter()
                .map(|chunk| {
                    let mut count: f32 = 0.0;
                    let mut avg = 0.0;
                    for &val in chunk {
                        avg += val;
                        count += 1.0;
                    }
                    avg / count
                }) //Average each chunk
                .minmax()
                .into_option()
                .unwrap_or((0.0, 0.0))
        }

        //Try and find a rough range that the frame info values fall into. The values are smoothed so that they don't change instantaneously, or include outliers
        let (smooth_delta_min, smooth_delta_max);
        {
            let span_calculate_approx_range = trace_span!(
                target: UI_TRACE_BUILD_INTERFACE,
                "calculate_delta_range",
                sharp_delta_min = Empty,
                sharp_delta_max = Empty,
                smooth_delta_min = Empty,
                smooth_delta_max = Empty,
            )
            .entered();
            let (sharp_delta_min, sharp_delta_max) =
                chunked_smooth_minmax(&deltas[0..info_range_end], self.scale_smoothing);

            // Update the local value, and then copy it across to the self value
            // let (&sharp_delta_min, &sharp_delta_max) = deltas[0..info_range_end] // Slice the area that we're going to be displaying, or else we calculate on the area that isn't visible
            //     .iter()
            //     .minmax()
            //     .into_option()
            //     .unwrap_or((&0.0, &0.0));

            smooth_delta_min = vek::Lerp::lerp(
                self.smooth_delta_min,
                sharp_delta_min,
                config.runtime.ui.frame_info_range_smooth_speed,
            );
            self.smooth_delta_min = smooth_delta_min;
            smooth_delta_max = vek::Lerp::lerp(
                self.smooth_delta_max,
                sharp_delta_max,
                config.runtime.ui.frame_info_range_smooth_speed,
            );
            self.smooth_delta_max = smooth_delta_max;

            span_calculate_approx_range.record("sharp_delta_min", sharp_delta_min);
            span_calculate_approx_range.record("sharp_delta_max", sharp_delta_max);
            span_calculate_approx_range.record("smooth_delta_min", smooth_delta_min);
            span_calculate_approx_range.record("smooth_delta_max", smooth_delta_max);
            span_calculate_approx_range.exit();
        }

        ui.plot_histogram(
            format!(
                "{:0>5.2} .. {:0>5.2} ms",
                smooth_delta_min, smooth_delta_max
            ),
            &deltas[0..info_range_end],
        )
        .overlay_text("ms/frame")
        .scale_min(smooth_delta_min)
        .scale_max(smooth_delta_max)
        .build();

        //Try and find a rough range that the frame info values fall into
        // These outer variables are the smoothed values (averaged across frames), inner ones are instantaneous
        let (smooth_fps_min, smooth_fps_max);
        {
            let span_calculate_approx_range = trace_span!(
                target: UI_TRACE_BUILD_INTERFACE,
                "calculate_delta_range",
                sharp_fps_min = Empty,
                sharp_fps_max = Empty,
                smooth_fps_min = Empty,
                smooth_fps_max = Empty,
            )
            .entered();

            let (sharp_fps_min, sharp_fps_max) =
                chunked_smooth_minmax(&fps[0..info_range_end], self.scale_smoothing);
            // Update the local value, and then copy it across to the self value
            smooth_fps_min = vek::Lerp::lerp(
                self.smooth_fps_min,
                sharp_fps_min,
                config.runtime.ui.frame_info_range_smooth_speed,
            );
            self.smooth_fps_min = smooth_fps_min;
            smooth_fps_max = vek::Lerp::lerp(
                self.smooth_fps_max,
                sharp_fps_max,
                config.runtime.ui.frame_info_range_smooth_speed,
            );
            self.smooth_fps_max = smooth_fps_max;

            span_calculate_approx_range.record("sharp_fps_min", sharp_fps_min);
            span_calculate_approx_range.record("sharp_fps_max", sharp_fps_max);
            span_calculate_approx_range.record("smooth_fps_min", smooth_fps_min);
            span_calculate_approx_range.record("smooth_fps_max", smooth_fps_max);
            span_calculate_approx_range.exit();
        }

        ui.plot_histogram(
            format!("{:0>6.2} .. {:>6.2} fps", smooth_fps_min, smooth_fps_max),
            &fps[0..info_range_end],
        )
        .overlay_text("frames/s")
        .scale_min(smooth_fps_min * 0.0)
        .scale_max(smooth_fps_max)
        .build();

        span_render_framerate_graph.exit();

        Ok(())
    }
}
