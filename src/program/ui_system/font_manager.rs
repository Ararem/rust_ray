//! Manages fonts for the UI system

use imgui::{FontConfig, FontId, FontSource, Ui};
use crate::config::ui_config::DEFAULT_FONT_SIZE;
use crate::helper::logging::event_targets;
use tracing::instrument;
use tracing::trace;

pub struct FontManager {
    /// Fonts available for the UI
    pub fonts: Vec<Font>,
}
impl FontManager {
    pub fn new() -> Self {
        FontManager{
            fonts: vec![
            Font{
                //JB Mono has a no-ligatures version, but we like ligatures so ignore that one
                name: "JetBrains Mono",
                variants: vec![
                    FontVariant{
                        name: "Regular",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Regular.ttf")
                    },
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
                        name: "Bold",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                    },
                    FontVariant{
                        name: "Extra Bold",
                        data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-ExtraBold.ttf")
                    }
                ]
            }
        ]}
    }

    #[instrument(level = "debug", skip_all)]
    pub fn add_fonts(&self, imgui: &mut imgui::Context) {
        // Fixed font size. Note imgui_winit_support uses "logical
        // pixels", which are physical pixels scaled by the devices
        // scaling factor. Meaning, 15.0 pixels should look the same size
        // on two different screens, and thus we do not need to scale this
        // value (as the scaling is handled by winit)
        let font_config = FontConfig {
            //TODO: Configure
            // Oversampling font helps improve text rendering at
            // expense of larger font atlas texture.
            oversample_h: 4,
            oversample_v: 4,
            // As imgui-glium-renderer isn't gamma-correct with
            // it's font rendering, we apply an arbitrary
            // multiplier to make the font a bit "heavier". With
            // default imgui-glow-renderer this is unnecessary.
            // rasterizer_multiply: 1.5,
            //Sets everything to default
            //Except the stuff we overrode before
            //SO COOOL!!
            ..FontConfig::default()
        };
        trace!("clearing builtin fonts");
        imgui.fonts().clear();
        for font in self.fonts.iter() {
            trace!("processing font {font}", font = font.name);
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
                    size = DEFAULT_FONT_SIZE
                )
                .into();
                imgui.fonts().add_font(&[FontSource::TtfData {
                    data: variant.data,
                    config: Some(FontConfig {
                        name: full_name,
                        ..font_config.clone()
                    }),
                    size_pixels: DEFAULT_FONT_SIZE,
                }]);
            }
        }

        //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
        trace!("building font atlas");
        // imgui.fonts().build_rgba32_texture();
        imgui.fonts().build_alpha8_texture();
    }

    /// Renders the font selector, and returns the selected font
    pub fn render_font_selector(&mut self, ui: &Ui, font: &mut FontId) {}
}
pub struct Font {
    /// Name of the base font, e.g. JetBrains Mono
    name: &'static str,
    /// Vec of font variants
    variants: Vec<FontVariant>,
}

/// A variant a font can have (i.e. bold, light, regular)
pub struct FontVariant {
    /// Name of the variant (i.e. "light")
    name: &'static str,
    /// Binary font data for this variant
    data: &'static [u8],
}
