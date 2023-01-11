use crate::config::compile_time::ui_config::{MAX_FONT_SIZE, MIN_FONT_SIZE};
use crate::config::read_config_value;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_display;
use crate::ui::build_ui_impl::UiItem;
use crate::ui::font_manager::FontManager;
use crate::FallibleFn;
use color_eyre::{Help, Report};
use imgui::{TreeNodeFlags, Ui};
use std::borrow::Cow::Borrowed;
use tracing::{debug, error, info, trace, trace_span, warn};

impl UiItem for FontManager {
    /// Renders the font selector, and returns the selected font
    fn render(&mut self, ui: &Ui, mut visible: bool) -> FallibleFn {
        //TODO: Move the validation code out from the UI code, and put it before the visible check
        let span_render_font_manager = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_font_manager").entered();
        // NOTE: We could get away with a lot of this code, but it's safer to have it, and more informative when something happens
        visible &= ui.collapsing_header("Font Manager", TreeNodeFlags::empty());
        if !visible {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "font manager collapsed");
            return Ok(());
        }

        trace!(target: UI_TRACE_BUILD_INTERFACE, "[Button] reload fonts list");
        if ui.button("Reload fonts list") {
            match self.reload_list_from_resources() {
                Ok(_) => info!(target: UI_DEBUG_GENERAL, "font list reloaded"),
                Err(err) => {
                    let report = err.wrap_err("could not reload fonts list from resources").note("called manually by user in font manager UI");
                    warn!(target: GENERAL_WARNING_NON_FATAL, report = format_report_display(&report));
                }
            }
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[Button] regenerate font atlas");
        if ui.button("Regenerate font atlas") {
            self.dirty = true;
        }

        // Whether the manager needs to rebuild the font next frame
        let dirty = &mut self.dirty;

        // # SELECTING BASE FONT
        let fonts = &mut self.fonts;
        let font_index = &mut self.selected_font_index;
        let fonts_len = fonts.len();

        if fonts_len == 0 {
            //Check we have at least one font, or else code further down fails (index out of bounds)
            ui.text_colored(read_config_value(|config| config.runtime.ui.colours.severity.warning), "No fonts loaded");
            trace!(target: UI_TRACE_BUILD_INTERFACE, "exiting early: no fonts (`fonts_len==0`)");
            return Ok(());
        }
        if *font_index >= fonts_len {
            /*
             Ensure font index is in bounds.
             Realistically should only happen when reloading fonts (was valid index for old list, now longer valid for new list), where it *should* be caught and fixed
             But better be safe
            */
            let clamped = fonts_len - 1;
            warn!(target: GENERAL_WARNING_NON_FATAL, "font_index ({font_index}) was >= fonts.len() ({fonts_len}), clamping ({clamped})");
            *font_index = clamped;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[combo] font selector");
        if ui.combo("Font", font_index, fonts, |f| Borrowed(&f.name)) {
            debug!(target: UI_DEBUG_USER_INTERACTION, "changed font to [{font_index}]: {font_name}", font_name = fonts[*font_index].name);
            *dirty = true;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[tooltip] font selector");
        if ui.is_item_hovered() {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "[hovered] font selector");
            ui.tooltip_text("Select a font to use for the user interface (UI)");
        }

        // # SELECTING FONT WEIGHT
        let weights = &mut fonts[*font_index].weights;
        let weight_index = &mut self.selected_weight_index;
        let weights_len = weights.len();

        if weights_len == 0 {
            ui.text_colored(read_config_value(|config| config.runtime.ui.colours.severity.warning), "(Bad) No weights loaded for the selected font.");
            /*
             * The way it's done is by getting the font and weight name from font file, and then placing the file into nested hashmaps (`fonts[base_font].insert(weight)`).
             * We should never get an empty font, since the entry for a font only ever gets created when we have to insert a weight and don't have a parent font entry already
             * If we get to here, something has gone seriously wrong
             */
            let report = Report::msg("had no weights loaded for the selected font")
                .note("this *REALLY* shouldn't happen (due to the internals of font loading and creation)\nperhaps some errors happened when loading the fonts?");
            error!(target: GENERAL_WARNING_NON_FATAL, report = format_report_display(&report));
            trace!(target: UI_TRACE_BUILD_INTERFACE, "exiting early: `weights.len() == 0`");
            return Ok(());
        }
        if *weight_index >= weights_len {
            // Ensure weights index is in bounds, this can fail when reloading fonts and/or changing base font (was valid index for old list, now longer valid for new list)
            let clamped = weights_len - 1;
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "weight_index ({weight_index}) was >= weights_len ({weights_len}), clamping ({clamped})"
            );
            *weight_index = clamped;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[combo] weight selector");
        if ui.combo("Weight", weight_index, weights, |v| Borrowed(&v.name)) {
            // don't need to update index, imgui does that automagically since we passed in a mut reference
            trace!(
                target: UI_DEBUG_USER_INTERACTION,
                "changed font weight to [{weight_index}]: {weight_name}",
                weight_name = weights[*weight_index].name
            );
            *dirty = true;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[tooltip] weight selector");
        if ui.is_item_hovered() {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "[hovered] weight selector");
            ui.tooltip_text("Customise the weight of the UI font (how bold it is)");
        }

        // # SELECTING FONT SIZE
        let size = &mut self.selected_size;
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[slider] font size");
        if ui.slider("Size (px)", MIN_FONT_SIZE, MAX_FONT_SIZE, size) {
            trace!(target: UI_DEBUG_USER_INTERACTION, "changed font size to {size} px");
            if *size < MIN_FONT_SIZE {
                warn!(target: GENERAL_WARNING_NON_FATAL, "font size ({size}) was < MIN_FONT_SIZE ({MIN_FONT_SIZE}), clamping");
                *size = MIN_FONT_SIZE;
            }
            if *size > MAX_FONT_SIZE {
                warn!(target: GENERAL_WARNING_NON_FATAL, "font size ({size}) was > MAX_FONT_SIZE ({MAX_FONT_SIZE}), clamping");
                *size = MAX_FONT_SIZE;
            }
            *dirty = true;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[tooltip] font size");
        if ui.is_item_hovered() {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "[hovered] font size");
            ui.tooltip_text("Change the size of the font (in logical pixels)");
        }
        span_render_font_manager.exit();

        Ok(())
    }
}
