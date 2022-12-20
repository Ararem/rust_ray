//! Manages fonts for the UI system

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Read;
use std::ops::Deref;
use Cow::Borrowed;

use color_eyre::eyre::Context;
use color_eyre::{eyre, Help, Report};
use fs_extra::*;
use imgui::{FontAtlas, FontConfig, FontId, FontSource, TreeNodeFlags, Ui};
use indoc::formatdoc;
use tracing::warn;
use tracing::{debug, debug_span, error, info, trace, trace_span};

use crate::config::resources_config::{
    FONTS_FILE_NAME_EXTRACTOR, FONTS_FILE_PATH_FILTER, FONTS_PATH,
};
use crate::config::ui_config::colours::COLOUR_ERROR;
use crate::config::ui_config::*;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_error;
use crate::resources::resource_manager::get_main_resource_folder_path;

#[derive(Debug)]
pub struct FontManager {
    /// Fonts available for the UI
    fonts: Vec<Font>,
    /// Index for which font we want to use (see [fonts])
    selected_font_index: usize,
    /// Index for which [FontWeight] from the selected font (see [font_index]) we want
    selected_weight_index: usize,
    /// Index for the selected font size (see [FONT_SIZES])
    selected_size: f32,
    /// The currently selected font's [FontId]
    current_font: Option<FontId>,
    /// Whether the font needs to be rebuilt because of a change
    dirty: bool,
}

impl FontManager {
    /// Reloads the list of available fonts, from the resources folder (in the build directory)
    pub fn reload_list_from_resources(&mut self) -> eyre::Result<()> {
        let span_reload_fonts_list = debug_span!(target: RESOURCES_DEBUG_LOAD, "reload_fonts_list");
        let fonts_directory_path = get_main_resource_folder_path()?.join(FONTS_PATH);

        debug!(
            target: RESOURCES_DEBUG_LOAD,
            "reloading fonts from resources folder {:?}", fonts_directory_path
        );
        let fonts_dir_content = dir::get_dir_content(&fonts_directory_path)
            .wrap_err("could not load fonts directory")
            .note(format!("Attempted to load from {:?}", fonts_directory_path))?;

        debug!(target: DATA_DEBUG_DUMP_OBJECT, size=fonts_dir_content.dir_size, directories=?fonts_dir_content.directories, files=?fonts_dir_content.files);

        let filter_regex = FONTS_FILE_PATH_FILTER.deref();
        debug!(target: DATA_DEBUG_DUMP_OBJECT, file_path_filter_regex=?filter_regex);
        let name_extractor_regex = FONTS_FILE_NAME_EXTRACTOR.deref();
        debug!(target: DATA_DEBUG_DUMP_OBJECT, font_name_extractor_regex=?name_extractor_regex);

        // We read the file into this buffer before we process it
        let mut font_data_buffer = Vec::with_capacity(512 * 1024 /*512kb default*/);

        // Nested hashmaps store data
        // First layer is [base font name]
        // Second layer contains [weight name] and font data
        let mut fonts: HashMap<&str, HashMap<&str, Vec<u8>>> = HashMap::new();
        debug_span!(target: RESOURCES_DEBUG_LOAD, "iter_font_dir").in_scope(||
            for file_path in fonts_dir_content.files.iter() {
                let span_internal_iter = trace_span!(target: FONT_MANAGER_TRACE_FONT_LOAD, "internal_iter", ?file_path);
                if !filter_regex.is_match(file_path) {
                    trace!(target: FONT_MANAGER_TRACE_FONT_LOAD, "skipping non-matching file path at {file_path}");
                    continue;
                }
                trace!(target: FONT_MANAGER_TRACE_FONT_LOAD, "reading matching file at {file_path}");

                let mut file = match fs::File::open(file_path) {
                    Ok(file) => file,
                    Err(err) => {
                        let report = Report::new(err)
                            .wrap_err(format!("was not able to open font file at {file_path}"));
                        warn!(target: RESOURCES_WARNING_NON_FATAL, ?report);
                        continue;
                    }
                };

                font_data_buffer.clear();
                match file.read_to_end(&mut font_data_buffer) {
                    Ok(_read) => {}
                    Err(error) => {
                        let report = Report::new(error).wrap_err(format!(
                            "could not read bytes from font file at {file_path}"
                        ));
                        warn!(target: RESOURCES_WARNING_NON_FATAL, ?report);
                        continue;
                    }
                }

                // Extract font names from the file path using Regex
                let mut base_font_name = "Unknown Fonts"; // Should be overwritten unless something goes wrong, this value is fallback
                let mut weight_name = file_path.as_str(); // Should be overwritten unless something goes wrong, this value is fallback
                // Try trim the file_path default value so it's not as long. Should always complete but just to be sure
                if let Some(pat) = fonts_directory_path.to_str() {
                    weight_name = weight_name
                        .trim_start_matches(pat)
                        .trim_start_matches(&['/', '\\']);
                } else {
                    trace!(target: FONT_MANAGER_TRACE_FONT_LOAD, "could not trim file path: could not convert base resources path to valid  UTF-8 [&str]")
                }
                for capture in name_extractor_regex.captures_iter(file_path) {
                    if let Some(_match) = capture.name("base_font_name") {
                        base_font_name = _match.as_str();
                    }
                    if let Some(_match) = capture.name("weight_name") {
                        weight_name = _match.as_str();
                    }
                }

                let base_font_ref = fonts
                    .entry(base_font_name)
                    .or_insert_with_key(|key|
                        {
                            trace!(target: FONT_MANAGER_TRACE_FONT_LOAD, "inserting HashMap entry for base font {}", key);
                            HashMap::new()
                        });

                trace!(target: FONT_MANAGER_TRACE_FONT_LOAD, base_font_name, weight_name, "inserting font into map");
                if let Some(old_data_buffer) = base_font_ref.insert(weight_name, font_data_buffer.clone()) {
                    warn!(target: RESOURCES_WARNING_NON_FATAL, "font entry already existed for {} @ {}", base_font_name, weight_name);
                    debug!(target: RESOURCES_WARNING_NON_FATAL, ?old_data_buffer);
                }
                span_internal_iter.exit();
            });

        // Now we have loaded file data, process into Font{} structs
        trace!(
            target: FONT_MANAGER_TRACE_FONT_LOAD,
            "clearing self fonts list"
        );
        self.fonts.clear();

        debug_span!(target: RESOURCES_DEBUG_LOAD, "load_fonts").in_scope(|| {
            for font_entry in fonts {
                let span_base_font_entry = trace_span!(
                    target: FONT_MANAGER_TRACE_FONT_LOAD,
                    format!("font_{}", font_entry.0).as_str()
                );
                debug!(target: DATA_DEBUG_DUMP_OBJECT, ?font_entry);

                let base_font_name = font_entry.0;
                let mut font = Font {
                    name: base_font_name.to_string(),
                    weights: vec![],
                };

                for weight_entry in font_entry.1 {
                    let weight_name = weight_entry.0;
                    trace!(
                        target: FONT_MANAGER_TRACE_FONT_LOAD,
                        "processing font {base_font_name} weight {weight_name}"
                    );
                    let data = weight_entry.1;

                    let weight = FontWeight {
                        name: weight_name.to_string(),
                        data,
                    };
                    font.weights.push(weight);
                }

                //Sort the fonts by their name
                font.weights
                    .sort_unstable_by(|w1, w2| w1.name.cmp(&w2.name));

                // Push the font once it's complete
                self.fonts.push(font);

                span_base_font_entry.exit();
            }
        });

        /*
        Now that we have a new list, make sure that our indices are still valid
        Also mark as dirty for rebuild, just in case

        Note on indices:
        Here's an example, pseudocode:
        i.e. old fonts is [5], index=4
        `reload()`
        index is 4, but list is now [4] (one font was removed)
        index isn't valid anymore, need to clamp to 3
        */
        trace_span!(target: FONT_MANAGER_TRACE_FONT_LOAD, "validate_indices").in_scope(||
            {
                let font_index = &mut self.selected_font_index;
                let fonts_len = self.fonts.len();
                if fonts_len == 0 {
                    warn!(target: GENERAL_WARNING_NON_FATAL, "font manager has no fonts after reloading");
                    return; // Closure
                }
                if *font_index >= fonts_len {
                    trace!(
                        target: FONT_MANAGER_TRACE_FONT_LOAD,
                        "had invalid font index: font_index ({font_index}) was >= fonts_len ({fonts_len}), clamping\nthis is fine, fonts list probably shrunk after reloading"
                    );
                    *font_index = fonts_len - 1;
                }

                let weight_index = &mut self.selected_weight_index;
                let weights_len = self.fonts[*font_index].weights.len();
                if weights_len == 0 {
                    warn!(target: GENERAL_WARNING_NON_FATAL, "font manager has no weights for font {}", self.fonts[*font_index].name);
                    return; // Closure
                }
                if *weight_index >= weights_len {
                    trace!(
                        target: FONT_MANAGER_TRACE_FONT_LOAD,
                        "had invalid weight index: weight_index ({weight_index}) was >= weights_len ({weights_len}), clamping\nthis is fine, fonts list probably shrunk after reloading"
                    );
                    *weight_index = weights_len - 1;
                }
            });

        span_reload_fonts_list.exit();
        return Ok(());
    }

    pub fn new() -> eyre::Result<Self> {
        let manager = FontManager {
            fonts: vec![],
            selected_font_index: 0,
            selected_weight_index: 0,
            selected_size: DEFAULT_FONT_SIZE,

            current_font: None,
            dirty: true,
        };
        return Ok(manager);
    }

    /// Rebuilds the font texture if required
    ///
    /// Return value when [`Ok`] is [`true`] if the font was rebuilt, otherwise [`false`] if it was not rebuilt.
    ///
    /// Note:
    /// If this returns `Ok(true)`, you ***MUST*** call `renderer.reload_font_texture(imgui_context)` or the app will crash
    pub fn rebuild_font_if_needed(&mut self, font_atlas: &mut FontAtlas) -> eyre::Result<bool> {
        // Don't need to update if we already have a font and we're not dirty
        if !self.dirty && self.current_font != None {
            return Ok(false);
        }
        let span_rebuild_font = debug_span!(target: UI_DEBUG_GENERAL, "rebuild_font").entered();

        debug!(target: UI_DEBUG_GENERAL, "clearing font atlas");
        font_atlas.clear();

        let fonts = &mut self.fonts;
        let font_index = &mut self.selected_font_index;

        if fonts.len() == 0 {
            let error = Report::msg("could not rebuild font: not fonts loaded")
                .suggestion("try ensuring that [reload_list_from_resources] has been called, and that it loaded fonts correctly (completes without error)");
            return Err(error);
        }

        // Check our indices are in the correct range
        *font_index = (*font_index).clamp(0usize, fonts.len() - 1usize);
        let base_font = &mut fonts[*font_index];

        let weights = &mut base_font.weights;
        let weight_index = &mut self.selected_weight_index;
        *weight_index = (*weight_index).clamp(0usize, weights.len() - 1usize);
        let weight = &weights[*weight_index];

        let size = &mut self.selected_size;

        // Important: having a negative size is __BAD__
        *size = (*size).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);

        debug!(
            target: UI_DEBUG_GENERAL,
            "building font {font_name} ({weight}) @ {size}px",
            font_name = base_font.name,
            weight = weight.name,
            size = *size
        );
        debug!(target: DATA_DEBUG_DUMP_OBJECT, data = ?weight.data);

        let full_name = format!(
            "{name} - {weight} ({size}px)",
            name = base_font.name,
            weight = weight.name
        )
        .into();
        //TODO: What happens if a font file has invalid font data (or isn't a font file)
        let font_id = font_atlas.add_font(&[FontSource::TtfData {
            data: &weight.data,
            config: Some(FontConfig {
                name: full_name,
                ..base_font_config()
            }),
            size_pixels: *size,
        }]);
        self.current_font = Some(font_id);

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        debug!(target: UI_DEBUG_GENERAL, "building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();

        self.dirty = false;

        span_rebuild_font.exit();
        return Ok(true);
    }

    pub fn get_font_id(&mut self) -> eyre::Result<&FontId> {
        //TODO: Better error handling (actually try to get the index then fail, rather than failing early - we might be wrong)

        return match &self.current_font {
            Some(font) => Ok(font),
            None => Err(
                Report::msg("could not get [FontId]: self.current_font was [None];")
                    .note(formatdoc! {r"
                    `self.current_font` should be set when the font atlas is rebuilt.
                    It should be set by `rebuild_font_if_needed()`, so ensure that it gets called (before `get_font_id()`) and completes successfully
                    "})
                    .suggestion("ensure the font and atlas are built by calling `rebuild_font_if_needed()` before attempting to get the font")
            ),
        };
    }

    /// Renders the font selector, and returns the selected font
    pub fn render_font_manager(&mut self, ui: &Ui) {
        let span_render_font_manager = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "render_font_manager");
        // NOTE: We could get away with a lot of this code, but it's safer to have it, and more informative when something happens
        if !(ui.collapsing_header("Font Manager", TreeNodeFlags::empty())) {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "font manager collapsed");
            return;
        }

        trace!(
            target: UI_TRACE_BUILD_INTERFACE,
            "[Button] reload fonts list"
        );
        if ui.button("Reload fonts list") {
            match self.reload_list_from_resources() {
                Ok(_) => info!(target: UI_DEBUG_GENERAL, "font list reloaded"),
                Err(err) => {
                    let report = err
                        .wrap_err("could not reload fonts list from resources")
                        .note("called manually by user in font manager UI");
                    warn!(
                        target: GENERAL_WARNING_NON_FATAL,
                        report = format_error(&report)
                    );
                }
            }
        }

        // Whether the manager needs to rebuild the font next frame
        let dirty = &mut self.dirty;

        // # SELECTING BASE FONT
        let fonts = &mut self.fonts;
        let font_index = &mut self.selected_font_index;
        let fonts_len = fonts.len();

        if fonts_len == 0 {
            //Check we have at least one font, or else code further down fails (index out of bounds)
            ui.text_colored(COLOUR_ERROR, "No fonts loaded");
            trace!(
                target: UI_TRACE_BUILD_INTERFACE,
                "exiting early: no fonts (`fonts_len==0`)"
            );
            return;
        }
        if *font_index >= fonts_len {
            /*
             Ensure font index is in bounds.
             Realistically should only happen when reloading fonts (was valid index for old list, now longer valid for new list), where it *should* be caught and fixed
             But better be safe
            */
            let clamped = fonts_len - 1;
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "font_index ({font_index}) was >= fonts.len() ({fonts_len}), clamping ({clamped})"
            );
            *font_index = clamped;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[combo] font selector");
        if ui.combo("Font", font_index, fonts, |f| Borrowed(&f.name)) {
            debug!(
                target: UI_DEBUG_USER_INTERACTION,
                "changed font to [{font_index}]: {font_name}",
                font_name = fonts[*font_index].name
            );
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
            ui.text_colored(
                COLOUR_ERROR,
                "(Bad) No weights loaded for the selected font.",
            );
            /*
             * The way it's done is by getting the font and weight name from font file, and then placing the file into nested hashmaps (`fonts[base_font].insert(weight)`).
             * We should never get an empty font, since the entry for a font only ever gets created when we have to insert a weight and don't have a parent font entry already
             * If we get to here, something has gone seriously wrong
             */
            let report = Report::msg("had no weights loaded for the selected font")
                .note("this *REALLY* shouldn't happen (due to the internals of font loading and creation)\nperhaps some errors happened when loading the fonts?");
            error!(
                target: GENERAL_WARNING_NON_FATAL,
                report = format_error(&report)
            );
            trace!(
                target: UI_TRACE_BUILD_INTERFACE,
                "exiting early: `weights.len() == 0`"
            );
            return;
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
                warn!(target: GENERAL_WARNING_NON_FATAL,"font size ({size}) was > MAX_FONT_SIZE ({MAX_FONT_SIZE}), clamping");
                *size = MAX_FONT_SIZE;
            }
            *dirty = true;
        }
        trace!(target: UI_TRACE_BUILD_INTERFACE, "[tooltip] font size");
        if ui.is_item_hovered() {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "[hovered] font size");
            ui.tooltip_text("Change the size of the font (in logical pixels)");
        }
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    pub(crate) name: String,
    /// Vec of font weights
    pub(crate) weights: Vec<FontWeight>,
}

/// A weight a font can have (i.e. bold, light, regular)
#[derive(Debug, Clone)]
pub struct FontWeight {
    /// Name of the weight (i.e. "light")
    pub(crate) name: String,
    /// Binary font data for this weight
    pub(crate) data: Vec<u8>,
}
