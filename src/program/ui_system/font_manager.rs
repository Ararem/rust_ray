//! Manages fonts for the UI system

use crate::config::ui_config::{base_font_config, DEFAULT_FONT_SIZE, RENDERED_FONT_SIZE};
use crate::helper::logging::event_targets;
use crate::helper::logging::event_targets::UI_SPAMMY;
use color_eyre::eyre::eyre;
use imgui::{Context, FontAtlasRef, FontAtlasRefMut, FontConfig, FontId, FontSource, FontStackToken, Ui};
use std::env::var;
use color_eyre::eyre;
use tracing::trace;
use tracing::{instrument, warn};

#[derive(Debug, Clone)]
pub struct FontManager {
    /// Fonts available for the UI
    fonts: Vec<Font>,
    /// Index for which font we want to use (see [fonts])
    selected_font_index: usize,
    /// Index for which [FontVariant] from the selected font (see [font_index]) we want
    selected_variant_index: usize,
    /// Size in pixels for the selected font
    selected_font_size: f32, //TODO: Implement sizing
    /// Flag for if the [cached_font_id] is dirty (out of sync with the target selected values)
    dirty: bool,
    ///
    font_ids: Vec<Vec<FontId>>,
}

impl FontManager {
    pub fn new() -> Self {
        let manager = FontManager{
            fonts: vec![
            Font{
                //JB Mono has a no-ligatures version, but we like ligatures so ignore that one
                name: "JetBrains Mono",
                variants: vec![
                    FontVariant{
                        name: "Thin",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Thin.ttf")
                    },
                    FontVariant{
                        name: "Extra Light",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-ExtraLight.ttf")
                    },
                    FontVariant{
                        name: "Light",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Light.ttf")
                    },
                    FontVariant{
                        name: "Regular",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Regular.ttf")
                    },
                    FontVariant{
                        name: "Bold",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                    },
                    FontVariant{
                        name: "Extra Bold",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-ExtraBold.ttf")
                    }
                ]
            }
        ],
        // Indices corresponding to the default font, in this case JB Mono @ Regular
        selected_font_index:0,
        selected_variant_index:3,
        selected_font_size: DEFAULT_FONT_SIZE,

        dirty: true,
        font_ids: vec![]
        };
        trace!("new font manager instance initialised with {} fonts and {} variants", manager.fonts.len(), manager.fonts.iter().flat_map(|font| &font.variants).count());
        return manager;
    }

    #[instrument(skip_all)]
    pub fn build_fonts(&mut self, font_atlas: &mut FontAtlasRefMut) {
        trace!("clearing builtin fonts");
        font_atlas.clear();

        for (font_index,font) in self.fonts.iter().enumerate() {
            trace!("processing font {font}", font = font.name);
            self.font_ids.insert(self.selected_font_index, vec![]);
            for (variant_index,variant) in font.variants.iter().enumerate() {
                trace!("processing variant {variant}", variant = variant.name);
                trace!(
                    target: event_targets::DATA_DUMP,
                    "font data is {:?}",
                    variant.data
                );

                let full_name = format!(
                    "{name} - {variant} ({size}px)",
                    name = font.name,
                    variant = variant.name,
                    size = RENDERED_FONT_SIZE
                )
                .into();
                let font_id = font_atlas.add_font(&[FontSource::TtfData {
                    data: variant.data,
                    config: Some(FontConfig {
                        name: full_name,
                        ..base_font_config()
                    }),
                    size_pixels: RENDERED_FONT_SIZE,
                }]);
                self.font_ids[font_index]
                    .insert(variant_index, font_id);
            }
        }

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();
    }

    pub fn get_font_id(&mut self) -> eyre::Result<&FontId>{
        //TODO: Better error handling (actually try to get the index then fail, rather than failing early - we might be wrong)
        //Check that we have at least one FontId stored as a fallback
        if self.font_ids.len() == 0 {
            warn!("cannot update selected font: font_ids is empty");
            return Err(eyre!("cannot update selected font: font_ids.len() == 0"));
        } else if self.font_ids[0].len() == 0 {
            warn!("cannot update selected font: font_ids[0] is empty");
            return Err(eyre!("cannot update selected font: font_ids[0].len() == 0"));
        }

        let id;
        {
            let font_index = &mut self.selected_font_index;
            let variant_index = &mut self.selected_variant_index;
            let font_len = self.font_ids.len();
            if *font_index >= font_len {
                warn!("selected font index {font_index} was out of bounds for fonts size {font_len}. Setting to 0");
                *font_index = 0;
            }
            let variants = &self.font_ids[*font_index];
             let variant_len = variants.len();
            if *variant_index>= variant_len {
                warn!("selected variant index {variant_index} was out of bounds for variants vec size {variant_len}. Setting to 0");
                *variant_index = 0;
            }
            id = &variants[*variant_index];
        }

        return Ok(id);
    }

    /// Renders the font selector, and returns the selected font
    pub fn render_font_selector(&mut self, ui: &Ui) {
        ui.text("FONT SELECTOR GOES HERE");

        let font_index = self.selected_font_index;
        let variant_index = self.selected_variant_index;
        //Do nothing if we aren't making any changes
        if self.selected_font_index == font_index && self.selected_variant_index == variant_index {
            return self.dirty = false;
        }
        self.selected_font_index = font_index;
        self.selected_variant_index = variant_index;
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    name: &'static str,
    /// Vec of font variants
    variants: Vec<FontVariant>,
}

/// A variant a font can have (i.e. bold, light, regular)
#[derive(Debug, Copy, Clone)]
pub struct FontVariant {
    /// Name of the variant (i.e. "light")
    name: &'static str,
    /// Binary font data for this variant
    data: &'static [u8],
}
