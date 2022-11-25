//! Manages fonts for the UI system

use crate::config::ui_config::{base_font_config, DEFAULT_FONT_SIZE, RENDERED_FONT_SIZE};
use crate::helper::logging::event_targets;
use crate::helper::logging::event_targets::UI_SPAMMY;
use color_eyre::eyre::eyre;
use imgui::{Context, FontAtlasRef, FontAtlasRefMut, FontConfig, FontId, FontSource, Ui};
use std::env::var;
use tracing::instrument;
use tracing::trace;

#[derive(Debug, Clone)]
pub struct FontManager {
    /// Fonts available for the UI
    fonts: Vec<Font>,
    /// Index for which font we want to use (see [fonts])
    selected_font_index: usize,
    /// Index for which [FontVariant] from the selected font (see [font_index]) we want
    selected_variant_index: usize,
    /// Size in pixels for the selected font
    selected_font_size: f32,
    /// Flag for if the [cached_font_id] is dirty (out of sync with the target selected values)
    dirty: bool,
    ///
    font_ids: Vec<Vec<FontId>>,
}

impl FontManager {
    pub fn get_current_font(&self) -> color_eyre::Result<FontId> {
        match self.cached_font_id {
            Some(font) => return Ok(font),
            None => return Err(eyre!("attempted to get current font when no value was stored (bad) - rebuild_font_if_necessary() should have already been called"))
        }
    }

    pub fn new() -> Self {
        FontManager{
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
        selected_font_size: 20f32,

        dirty: true,
        font_ids: vec![]
        }
    }

    #[instrument(skip_all)]
    pub fn build_fonts(&mut self, font_atlas: &mut FontAtlasRefMut) {
        trace!("clearing builtin fonts");
        font_atlas.clear();

        for font in self.fonts.iter() {
            trace!("processing font {font}", font = font.name);
            self.font_ids.insert(self.selected_font_index, vec![]);
            for variant in font.variants.iter() {
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
                self.font_ids[self.selected_font_index]
                    .insert(self.selected_variant_index, font_id);
            }
        }

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // font_atlas.build_rgba32_texture();
        font_atlas.build_alpha8_texture();
    }

    pub fn update_font(&mut self, font_atlas: &mut FontAtlasRefMut) {
        font_atlas.tex_id; // <============!!!!!!
        if !self.dirty && self.cached_font_id != None {
            trace!(target: UI_SPAMMY, "no need to rebuild font");
            return;
        }
        trace!("cached font dirty or non-existent, rebuilding");
        let base_font = &self.fonts[self.selected_font_index];
        let font = &base_font.variants[self.selected_variant_index];
        let font_size = self.selected_font_size;
        trace!(
            "font is [{f_index}] ({f_name}), variant [{v_index}] ({v_name}), size {size}",
            f_index = self.selected_font_index,
            f_name = base_font.name,
            v_index = self.selected_variant_index,
            v_name = font.name,
            size = font_size
        );

        trace!(
            target: event_targets::DATA_DUMP,
            "font data is {:?}",
            font.data
        );

        // trace!("clearing builtin fonts");
        // imgui.fonts().clear();

        let full_name = format!(
            "{name} - {variant} ({size}px)",
            name = base_font.name,
            variant = font.name,
            size = font_size
        )
        .into();

        let font_id = font_atlas
            .add_font(&[FontSource::TtfData {
                data: font.data,
                config: Some(FontConfig {
                    name: full_name,
                    ..base_font_config()
                }),
                size_pixels: font_size,
            }])
            .into();

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // imgui.fonts().build_rgba32_texture();
        font_atlas.build_alpha8_texture();

        trace!("done rebuilding font atlas");
        self.cached_font_id = font_id;
        self.dirty = false;
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
