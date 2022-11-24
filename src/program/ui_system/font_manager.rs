//! Manages fonts for the UI system

use imgui::{FontConfig, FontSource};
// Font family
// Font style (bold, light, etc)
// Font size
use crate::helper::logging::event_targets;
use lazy_static::lazy_static;
use tracing::{debug, instrument, span, trace, trace_span, Level};
lazy_static! {
    /// Fonts available for the UI
    pub static ref FONTS: Vec<Font> = vec![
        Font{
            //JB Mono has a no-ligatures version, but we like ligatures so ignore that one
            name: "JetBrains Mono",
            variants: vec![
                FontVariant{
                    name: "Bold",
                    data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                },
                FontVariant{
                    name: "Bold Italic",
                    data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-BoldItalic.ttf")
                },
                FontVariant{
                    name: "Extra Bold",
                    data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-ExtraBold.ttf")
                },
                FontVariant{
                    name: "Extra Bold Italic",
                    data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-ExtraBoldItalic.ttf")
                },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")let CALLSITE// FontVariant{let CALLSITE
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // },
                // FontVariant{
                //     name: "Bold",
                //     data: include_bytes!("../../resources/fonts/JetBrains Mono v2.242/fonts/ttf/JetBrainsMono-Bold.ttf")
                // }
            ]
        }
    ];
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

#[instrument(level = "debug", skip(imgui))]
pub fn add_fonts(imgui: &mut imgui::Context) {
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
    for font in FONTS.iter() {
        trace!("processing font {font}", font = font.name);
        for variant in font.variants.iter() {
            trace!("processing variant {variant}", variant = variant.name);
            trace!(
                target: event_targets::DATA_DUMP,
                "font data is {:?}",
                variant.data
            );

            for font_size in vec![12f32, 16f32, 24f32, 32f32, 48f32, 64f32] {
                imgui.fonts().add_font(&[FontSource::TtfData {
                    data: variant.data,
                    config: Some(FontConfig {
                        name: format!(
                            "{name} - {variant} ({size}px)",
                            name = font.name,
                            variant = variant.name,
                            size = font_size
                        )
                        .into(),
                        ..font_config.clone()
                    }),
                    size_pixels: font_size,
                }]);
            }
        }
    }

    //Not sure what the difference is between RGBA32 and Alpha8 atlases, other than channel count
    trace!("building font atlas");
    // imgui.fonts().build_rgba32_texture();
    imgui.fonts().build_alpha8_texture();
}
/*


   //TODO: Proper resource manager
   {
       debug!("adding fonts");

       // Fixed font size. Note imgui_winit_support uses "logical
       // pixels", which are physical pixels scaled by the devices
       // scaling factor. Meaning, 15.0 pixels should look the same size
       // on two different screens, and thus we do not need to scale this
       // value (as the scaling is handled by winit)
       let font_size = 50.0;
       let font_config = FontConfig {
           //TODO: Configure
           // Oversampling font helps improve text rendering at
           // expense of larger font atlas texture.
           oversample_h: 4,
           oversample_v: 4,
           size_pixels: font_size,
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

       //TODO: Multiple families of a font

       macro_rules! font_and_families {
           () => {};
       }

       macro_rules! font {
           ($name:literal, $path:literal) => {{
               //Yes i did write these all by hand
               // font_sized!($name, 8f32,  $path);
               // font_sized!($name, 10f32, $path);
               font_sized!($name, 12f32, $path);
               font_sized!($name, 14f32, $path);
               font_sized!($name, 16f32, $path);
               // font_sized!($name, 18f32, $path);
               font_sized!($name, 20f32, $path);
               // font_sized!($name, 22f32, $path);
               font_sized!($name, 24f32, $path);
               // font_sized!($name, 26f32, $path);
               // font_sized!($name, 28f32, $path);
               // font_sized!($name, 30f32, $path);
               font_sized!($name, 32f32, $path);
               // font_sized!($name, 34f32, $path);
               // font_sized!($name, 36f32, $path);
               // font_sized!($name, 38f32, $path);
               font_sized!($name, 40f32, $path);
               // font_sized!($name, 42f32, $path);
               // font_sized!($name, 44f32, $path);
               // font_sized!($name, 46f32, $path);
               // font_sized!($name, 48f32, $path);
               // font_sized!($name, 50f32, $path);
               // font_sized!($name, 52f32, $path);
               // font_sized!($name, 54f32, $path);
               font_sized!($name, 56f32, $path);
               // font_sized!($name, 58f32, $path);
               // font_sized!($name, 60f32, $path);
               // font_sized!($name, 62f32, $path);
               // font_sized!($name, 64f32, $path);
           }};
       }
       macro_rules! font_sized {
           //TODO: Make the macro accept a path not just any old expression
           ($name:literal, $size:expr, $path:literal) => {{
               imgui.fonts().add_font(&[FontSource::TtfData {
                   data: include_bytes!($path),
                   config: Some(FontConfig {
                       name: format!("{name} ({size}px)", name = $name, size = $size).into(),
                       ..font_config.clone()
                   }),
                   size_pixels: $size,
               }]);
           }};
       }

       imgui.fonts().clear();
       font!(
           "Jetbrains Mono v2.242",
           "..\\..\\resources\\fonts\\JetBrains Mono v2.242\\fonts\\ttf\\JetBrainsMonoNL-Medium.ttf"
       );

       imgui.fonts().build_rgba32_texture();
       trace!("added fonts");
   }

*/
