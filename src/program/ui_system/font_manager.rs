//! Manages fonts for the UI system

use Cow::Borrowed;
use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use fs_extra::*;
use path_clean::PathClean;
use color_eyre::{eyre, Help, Report};
use color_eyre::eyre::{ErrReport, eyre};
use imgui::{FontAtlasRefMut, FontConfig, FontId, FontSource, FontStackToken, InputFloat, InputTextFlags, ItemHoveredFlags, TreeNodeFlags, Ui};
use tracing::{debug, error, info, trace, trace_span};
use tracing::{instrument, warn};
use crate::config::resources_config::FONTS_PATH;
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
    #[instrument(skip_all, level="trace")]
    pub fn reload_list_from_resources(&mut self) -> eyre::Result<()> {
        let path =
            get_main_resource_folder_path()?.join(FONTS_PATH);

        debug!("reloading fonts from resources folder {:?}", &path);
        let directory;
        match dir::get_dir_content(&path) {
            Err(e) => {
                let error = Report::wrap_err(e.into(), format!("could not load fonts directory {:?}", &path));
                error!("{error:#}");
                return Err(error);
            }
            Ok(dir) => directory = dir,
        }

        debug!(target:DATA_DUMP,"fonts folder directories: {:#?}", directory.directories);
        debug!(target:DATA_DUMP,"fonts folder files:{:#?}", directory.files);

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

        let _ = trace_span!("rebuild_font_if_needed").entered();
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
        let weight = weights[*weight_index];

        let size = &self.selected_size;

        trace!("processing font {font_name} ({weight}) @ {size}px", font_name = base_font.name, weight = weight.name);
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
            data: weight.data,
            config: Some(FontConfig {
                name: full_name,
                ..base_font_config()
            }),
            size_pixels: *size,
        }]);
        self.current_font = Some(font_id);

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();

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
                self.reload_list_from_resources();
            }
            if self.fonts.len() == 0 {
                ui.text_colored(ERROR, "No fonts loaded");
            } else {
                if ui.combo("Font", &mut self.selected_font_index, &self.fonts, |f| Borrowed(f.name)) {
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
                        |v| Borrowed(v.name),
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

#[derive(Debug)]
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    pub(crate) name: &'static str,
    /// Vec of font weights
    pub(crate) weights: &'static [FontWeight],
}

/// A weight a font can have (i.e. bold, light, regular)
#[derive(Debug, Copy, Clone)]
pub struct FontWeight {
    /// Name of the weight (i.e. "light")
    pub(crate) name: &'static str,
    /// Binary font data for this weight
    pub(crate) data: &'static [u8],
}
