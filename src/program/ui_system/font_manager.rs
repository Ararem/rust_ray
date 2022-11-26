//! Manages fonts for the UI system

use crate::config::ui_config::{
    base_font_config, BUILTIN_FONTS, DEFAULT_FONT_INDEX, DEFAULT_FONT_SIZE_INDEX,
    DEFAULT_FONT_VARIANT_INDEX, FONT_SIZES,
};
use crate::helper::logging::event_targets;
use color_eyre::eyre;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use glium::vertex::MultiVerticesSource;
use glium::CapabilitiesSource;
use imgui::FontAtlasRef::Owned;
use imgui::{FontAtlasRef, FontAtlasRefMut, FontConfig, FontId, FontSource, FontStackToken, Ui};
use std::borrow::{Borrow, Cow};
use tracing::{error, trace};
use tracing::{instrument, warn};
use tracing_subscriber::fmt::format;
use Cow::Borrowed;

#[derive(Debug, Clone)]
pub struct FontManager {
    /// Fonts available for the UI
    fonts: &'static [Font],
    /// Index for which font we want to use (see [fonts])
    selected_font_index: usize,
    /// Index for which [FontVariant] from the selected font (see [font_index]) we want
    selected_variant_index: usize,
    /// Index for the selected font size (see [FONT_SIZES])
    selected_size_index: usize,
    /// Flag for if the [cached_font_id] is dirty (out of sync with the target selected values)
    dirty: bool,
    /// [FontId]'s for the built [fonts]. Indexing order is [selected_font_index] -> [selected_variant_index] -> [selected_size_index]
    font_ids: Vec<Vec<Vec<FontId>>>,
}

impl FontManager {
    pub fn new() -> Self {
        let manager = FontManager {
            fonts: BUILTIN_FONTS,
            selected_font_index: DEFAULT_FONT_INDEX,
            selected_variant_index: DEFAULT_FONT_VARIANT_INDEX,
            selected_size_index: DEFAULT_FONT_SIZE_INDEX,

            dirty: true,
            font_ids: vec![],
        };
        trace!(
            "new font manager instance initialised with {} fonts, {} variants",
            manager.fonts.len(),
            (*manager.fonts)
                .iter()
                .flat_map(|font| (*font.variants).into_iter())
                .count()
        );
        return manager;
    }

    #[instrument(skip_all)]
    pub fn build_fonts(&mut self, font_atlas: &mut FontAtlasRefMut) {
        trace!("clearing builtin fonts");
        font_atlas.clear();

        for (font_index, font) in self.fonts.iter().enumerate() {
            trace!("processing font {font}", font = font.name);
            self.font_ids.insert(font_index, vec![]);
            for (variant_index, variant) in font.variants.iter().enumerate() {
                trace!("processing variant {variant}", variant = variant.name);
                trace!(
                    target: event_targets::DATA_DUMP,
                    "font data is {:?}",
                    variant.data
                );
                self.font_ids[font_index].insert(variant_index, vec![]);

                for (size_index, size) in FONT_SIZES.iter().enumerate() {
                    trace!("processing size {size}px");

                    let full_name = format!(
                        "{name} - {variant} ({size}px)",
                        name = font.name,
                        variant = variant.name
                    )
                    .into();
                    let font_id = font_atlas.add_font(&[FontSource::TtfData {
                        data: variant.data,
                        config: Some(FontConfig {
                            name: full_name,
                            ..base_font_config()
                        }),
                        size_pixels: *size,
                    }]);
                    self.font_ids[font_index][variant_index].insert(size_index, font_id);
                }
            }
        }

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();
    }

    pub fn get_font_id(&mut self) -> eyre::Result<&FontId> {
        //TODO: Better error handling (actually try to get the index then fail, rather than failing early - we might be wrong)
        //Check that we have at least one FontId stored as a fallback
        if self.font_ids.len() == 0 {
            error!("cannot update selected font: font_ids is empty (`font_ids.len() == 0`)");
            return Err(eyre!(
                "cannot update selected font: font_ids is empty (`font_ids.len() == 0`)"
            ));
        } else if self.font_ids[0].len() == 0 {
            error!("cannot update selected font: variants is empty (`font_ids[0].len() == 0`)");
            return Err(eyre!(
                "cannot update selected font: variants is empty (`font_ids[0].len() == 0`)"
            ));
        } else if self.font_ids[0][0].len() == 0 {
            error!(
                "cannot update selected font: sizes is empty (`self.font_ids[0][0].len() == 0`)"
            );
            return Err(eyre!(
                "cannot update selected font: sizes is empty (`self.font_ids[0][0].len() == 0`)"
            ));
        }

        let id;
        {
            let font_index = &mut self.selected_font_index;
            let ids_by_font = &self.font_ids;
            let fonts_len = ids_by_font.len();
            if *font_index >= fonts_len {
                warn!("selected font index {font_index} was out of bounds for fonts len {fonts_len}. Setting to 0");
                *font_index = 0;
            }

            let variant_index = &mut self.selected_variant_index;
            let ids_by_variant = &ids_by_font[*font_index];
            let variants_len = ids_by_variant.len();
            if *variant_index >= variants_len {
                warn!("selected variant index {variant_index} was out of bounds for variants vec len {variants_len}. Setting to 0");
                *variant_index = 0;
            }

            let size_index = &mut self.selected_size_index;
            let ids_by_size = &ids_by_variant[*variant_index];
            let sizes_len = ids_by_size.len();
            if *size_index >= sizes_len {
                warn!("selected size index {size_index} was out of bounds for sizes vec len {sizes_len}. Setting to 0");
                *size_index = 0;
            }
            id = &ids_by_size[*size_index];
        }

        return Ok(id);
    }

    /// Renders the font selector, and returns the selected font
    pub fn render_font_selector(&mut self, ui: &Ui) {
        ui.text("FONT SELECTOR GOES HERE");

        ui.combo("Font", &mut self.selected_font_index, &self.fonts, |f| {
            Borrowed(f.name)
        });
        ui.combo(
            "Variant",
            &mut self.selected_variant_index,
            &self.fonts[self.selected_font_index].variants,
            |v| Borrowed(v.name),
        );
        ui.combo("Size", &mut self.selected_size_index, &FONT_SIZES, |s| {
            Cow::Owned(format!("{s} px"))
        });
    }
}

#[derive(Debug)]
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    pub(crate) name: &'static str,
    /// Vec of font variants
    pub(crate) variants: &'static [FontVariant],
}

/// A variant a font can have (i.e. bold, light, regular)
#[derive(Debug, Copy, Clone)]
pub struct FontVariant {
    /// Name of the variant (i.e. "light")
    pub(crate) name: &'static str,
    /// Binary font data for this variant
    pub(crate) data: &'static [u8],
}
