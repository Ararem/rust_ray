//! Manages fonts for the UI system

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::io::Read;
use std::ops::Deref;
use color_eyre::eyre::Context;
use color_eyre::{eyre, Help, Report};
use fs_extra::*;
use imgui::{FontAtlas, FontConfig, FontId, FontSource};
use indoc::formatdoc;
use nameof::name_of;
use tracing::warn;
use tracing::{debug, debug_span, trace, trace_span};

use crate::config::compile_time::resources_config::{
    FONTS_FILE_NAME_EXTRACTOR, FONTS_FILE_PATH_FILTER
};
use crate::config::read_config_value;
use crate::FallibleFn;
use crate::config::compile_time::ui_config::{MAX_FONT_SIZE, MIN_FONT_SIZE};
use crate::helper::logging::event_targets::*;
use crate::resources::resource_manager::get_main_resource_folder_path;

#[derive(Debug, Clone)]
pub struct FontManager {
    /// Fonts available for the UI
    pub (in crate::ui) fonts: Vec<Font>,
    /// Index for which font we want to use (see [fonts])
    pub (in crate::ui) selected_font_index: usize,
    /// Index for which [FontWeight] from the selected font (see [font_index]) we want
    pub (in crate::ui) selected_weight_index: usize,
    /// Index for the selected font size (see [FONT_SIZES])
    pub (in crate::ui) selected_size: f32,
    /// The currently selected font's [FontId]
    pub (in crate::ui) current_font: Option<FontId>,
    /// Whether the font needs to be rebuilt because of a change
    pub (in crate::ui) dirty: bool,
}

impl FontManager {
    /// Reloads the list of available fonts, from the resources folder (in the build directory)
    pub fn reload_list_from_resources(&mut self) -> FallibleFn {
        let span_reload_fonts_list =
            debug_span!(target: RESOURCES_DEBUG_LOAD, "reload_fonts_list").entered();

        /*
        A bug I noticed is, since the font selection is kinda random, the font doesn't actually get updated when the list is reloaded.
        This happens even when the selection changes. This is caused by [reload_list_from_resources] not actually marking the font manager as dirty, even though it modifies state
        Therefore, always mark as dirty (just to be safe)
        */
        self.dirty = true;

        let fonts_directory_path = get_main_resource_folder_path()?.join(read_config_value(|config| config.runtime.resources.fonts_path));

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
                let span_internal_iter = trace_span!(target: FONT_MANAGER_TRACE_FONT_LOAD, "internal_iter", ?file_path).entered();
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
                        .trim_start_matches(['/', '\\']);
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
                    let buffers_are_equal = old_data_buffer.eq(&font_data_buffer);
                    warn!(target: RESOURCES_WARNING_NON_FATAL, "font entry already existed for {} @ {}, old {equal} new", base_font_name, weight_name, equal=if buffers_are_equal {"=="} else {"!="});
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
                    "process_group",
                    base_font = font_entry.0
                )
                .entered();
                debug!(target: DATA_DEBUG_DUMP_OBJECT, ?font_entry);
                //
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
        Ok(())
    }

    pub fn new() -> eyre::Result<Self> {
        let manager = FontManager {
            fonts: vec![],
            selected_font_index: 0,
            selected_weight_index: 0,
            selected_size: 20f32, //TODO: Font size and weights in config

            current_font: None,
            dirty: true,
        };
        Ok(manager)
    }

    /// Rebuilds the font texture if required
    ///
    /// Return value when [`Ok`] is [`true`] if the font was rebuilt, otherwise [`false`] if it was not rebuilt.
    ///
    /// Note:
    /// If this returns `Ok(true)`, you ***MUST*** call `renderer.reload_font_texture(imgui_context)` or the app will crash
    pub fn rebuild_font_if_needed(&mut self, font_atlas: &mut FontAtlas) -> eyre::Result<bool> {
        // Don't need to update if we already have a font and we're not dirty
        if !self.dirty && self.current_font.is_some() {
            return Ok(false);
        }
        let span_rebuild_font = debug_span!(target: UI_DEBUG_GENERAL, "rebuild_font").entered();

        debug!(target: UI_DEBUG_GENERAL, "clearing font atlas");
        font_atlas.clear();

        let fonts = &mut self.fonts;
        let font_index = &mut self.selected_font_index;

        if fonts.is_empty() {
            let error = Report::msg("could not rebuild font: no fonts loaded (`fonts.is_empty() == true`)")
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
        let oversampling = read_config_value(|config| config.runtime.ui.font_oversampling);
        let font_id = font_atlas.add_font(&[FontSource::TtfData {
            data: &weight.data,
            config: Some(FontConfig {
                name: full_name,
                oversample_v: oversampling,
                oversample_h: oversampling,
                ..FontConfig::default()
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
        Ok(true)
    }

    pub fn get_font_id(&mut self) -> eyre::Result<&FontId> {
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
}

#[derive(Debug, Clone)]
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    pub(crate) name: String,
    /// Vec of font weights
    pub(crate) weights: Vec<FontWeight>,
}

/// A weight a font can have (i.e. bold, light, regular)
#[derive(Clone)]
pub struct FontWeight {
    /// Name of the weight (i.e. "light")
    pub(crate) name: String,
    /// Binary font data for this weight
    pub(crate) data: Vec<u8>,
}

/// Custom [Debug] impl for [FontWeight], doesn't print the actual contents of [FontWeight.data], but the length
impl Debug for FontWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(name_of!(type FontWeight))
            .field(name_of!(name in FontWeight), &self.name)
            .field("data.len", &self.data.len())
            .finish_non_exhaustive()
    }
}
