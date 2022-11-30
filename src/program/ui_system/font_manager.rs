//! Manages fonts for the UI system

use Cow::Borrowed;
use std::borrow::Cow;
use std::{env, fs};
use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use fs_extra::*;
use path_clean::PathClean;
use color_eyre::{eyre, Help, Report};
use color_eyre::eyre::{ErrReport, eyre};
use imgui::{FontAtlasRefMut, FontConfig, FontId, FontSource, FontStackToken, InputFloat, InputTextFlags, ItemHoveredFlags, TreeNodeFlags, Ui};
use regex::Regex;
use tracing::{debug, error, info, trace, trace_span};
use tracing::{instrument, warn};
use crate::config::resources_config::{FONTS_FILE_NAME_EXTRACTOR, FONTS_FILE_PATH_FILTER, FONTS_PATH};
use crate::config::ui_config::*;
use crate::config::ui_config::colours::ERROR;
use crate::helper::logging::event_targets;
use crate::helper::logging::event_targets::{DATA_DUMP, UI_USER_EVENT};
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
    #[instrument(skip_all, level = "trace")]
    pub fn reload_list_from_resources(&mut self) -> eyre::Result<()> {
        let fonts_directory =
            get_main_resource_folder_path()?.join(FONTS_PATH);

        debug!("reloading fonts from resources folder {:?}", &fonts_directory);
        let directory;
        match dir::get_dir_content(&fonts_directory) {
            Err(e) => {
                let error = Report::from(e).wrap_err(format!("could not load fonts directory {:?}", &fonts_directory));
                error!("{error}");
                return Err(error);
            }
            Ok(dir) => directory = dir,
        }

        debug!(target:DATA_DUMP,"fonts folder directories: {:#?}", &directory.directories);
        debug!(target:DATA_DUMP,"fonts folder files:{:#?}", &directory.files);

        //Visit each file in the tree
        let filter_regex = FONTS_FILE_PATH_FILTER.deref();
        trace!("file path filter for fonts is `{filter_regex:?}`");
        let name_extractor_regex = FONTS_FILE_NAME_EXTRACTOR.deref();
        trace!("font name extraction regex is `{name_extractor_regex:?}`");

        // We read the file into this buffer before we process it
        let mut font_data_buffer = Vec::with_capacity(512 * 1024 /*512kb default*/);

        // Nested hashmaps store data
        // First layer is [base font name]
        // Second layer contains [weight name] and font data
        let mut fonts: HashMap<&str, HashMap<&str, Vec<u8>>> = HashMap::new();
        for file_path in directory.files.iter() {
            if !filter_regex.is_match(file_path) {
                trace!("skipping non-matching file path at {file_path}");
                continue;
            }
            trace!("reading matching file at {file_path}");

            let mut file = match fs::File::open(file_path) {
                Ok(file) => { file },
                Err(err) => {
                    let report = Report::new(err)
                        .wrap_err(format!("was not able to open font file at {file_path}"));
                    warn!("{}", report);
                    continue;
                }
            };

            let bytes_read;
            font_data_buffer.clear();
            match file.read_to_end(&mut font_data_buffer) {
                Ok(_read) => bytes_read = _read,
                Err(error) => {
                    let report = Report::new(error)
                        .wrap_err(format!("could not read bytes from font file at {file_path}"));
                    warn!("{}", report);
                    continue;
                }
            }

            // Extract font names from the file path using Regex
            let mut base_font_name = "<Unknown Font>";
            let mut weight_name = "<Unknown Weight>";
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
                .or_insert_with(|| { HashMap::new() });

            base_font_ref
                .insert(weight_name, font_data_buffer.clone());
        }

        // Now we have loaded file data, process into Font{} structs
        for font_entry in fonts {
            let base_font_name = font_entry.0;
            trace!("processing base font {base_font_name}");
            let mut font = Font {
                name: base_font_name.to_string(),
                weights: vec![]
            };

            for weight_entry in font_entry.1 {
                let weight_name = weight_entry.0;
                trace!("processing font {base_font_name} weight {weight_name}");
                let data = weight_entry.1;

                let weight = FontWeight{
                    name: weight_name.to_string(),
                    data
                };
                font.weights.push(weight);
            }

            // Push the font once it's complete
            self.fonts.push(font);
        }

        return Ok(());
    }

    pub fn new() -> eyre::Result<Self> {
        let mut manager = FontManager {
            fonts: vec![],
            selected_font_index: 0,
            selected_weight_index: 0,
            selected_size: DEFAULT_FONT_SIZE,

            current_font: None,
            dirty: true,
        };
        manager.reload_list_from_resources()?;
        trace!(
            "new font manager instance initialised with {} fonts, {} weights",
            manager.fonts.len(),
            (*manager.fonts)
                .iter()
                .flat_map(|font| (*font.weights).into_iter())
                .count()
        );
        return Ok(manager);
    }

    pub fn rebuild_font_if_needed(&mut self, font_atlas: &mut FontAtlasRefMut) -> eyre::Result<()> {
        // Don't need to update if we already have a font and we're not dirty
        if !self.dirty && self.current_font != None {
            return Ok(());
        }

        let _guard = trace_span!("rebuild_font_if_needed").entered();
        // trace!("clearing builtin fonts");
        // font_atlas.clear();

        let fonts = &mut self.fonts;
        let font_index = &mut self.selected_font_index;

        if fonts.len() == 0 {
            let error = Report::msg("could not rebuild font: not fonts loaded")
                .note("try ensuring that [reload_list_from_resources] has been called, and that it loaded fonts correctly");
            return Err(error);
        }

        // Check our indices are in the correct range
        *font_index = (*font_index).clamp(0usize, fonts.len() - 1usize);
        let base_font = &mut fonts[*font_index];

        let weights = &mut base_font.weights;
        let weight_index = &mut self.selected_weight_index;
        *weight_index = (*weight_index).clamp(0usize, weights.len() - 1usize);
        let weight = &weights[*weight_index];

        let size = &self.selected_size;

        trace!("building font {font_name} ({weight}) @ {size}px", font_name = base_font.name, weight = weight.name);
        trace!(
                    target: event_targets::DATA_DUMP,
                    "font data is {:?}",
                    weight.data
                );

        let full_name = format!(
            "{name} - {weight} ({size}px)",
            name = base_font.name,
            weight = weight.name
        )
            .into();
        let font_id = font_atlas.add_font(&[FontSource::TtfData {
            data: &weight.data,
            config: Some(FontConfig {
                name: full_name,
                ..base_font_config()
            }),
            size_pixels: *size,
        }]);
        // self.current_font = Some(font_id);

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();

        _guard.exit();
        return Ok(());
    }

    pub fn get_font_id(&mut self) -> eyre::Result<&FontId> {
        //TODO: Better error handling (actually try to get the index then fail, rather than failing early - we might be wrong)

        return match &self.current_font {
            Some(font) => Ok(font),
            None => Err(eyre!("could not get [FontId]: self.current_font was [None]; should have already been set by [update_font_if_needed()]")),
        };
    }

    /// Renders the font selector, and returns the selected font
    pub fn render_font_selector(&mut self, ui: &Ui) {
        if ui.collapsing_header("Font Manager", TreeNodeFlags::empty()) {
            if ui.button("Reload fonts list") {
                match self.reload_list_from_resources(){
                    Ok(_) => info!("font list reloaded"),
                    Err(err) => {
                        let report = err.note("called manually by user in font manager UI");
                        warn!("{report}");
                    }
                }
            }
            if self.fonts.len() == 0 {
                ui.text_colored(ERROR, "No fonts loaded");
            } else {
                if ui.combo("Font", &mut self.selected_font_index, &self.fonts, |f| Borrowed(&f.name)) {
                    trace!(target: UI_USER_EVENT, "Changed font to [{new_font_index}]: {new_font}", new_font_index = self.selected_font_index, new_font = self.fonts[self.selected_font_index].name);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text("Select a font to use for the user interface (UI)");
                }
                if self.fonts[self.selected_font_index].weights.len() == 0 {
                    ui.text_colored(ERROR, "No weights loaded");
                } else {
                    ui.combo(
                        "Weight",
                        &mut self.selected_weight_index,
                        &self.fonts[self.selected_font_index].weights,
                        |v| Borrowed(&v.name),
                    );
                    if ui.is_item_hovered() {
                        ui.tooltip_text("Customise the weight of the UI font (how bold it is)");
                    }
                }
                InputFloat::new(
                    ui,
                    "Size (px)",
                    &mut self.selected_size
                ).build();
                if ui.is_item_hovered() {
                    ui.tooltip_text("Change the size of the font (in pixels)");
                }
            }
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
