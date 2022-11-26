use crate::program::ui_system::font_manager::{Font, FontVariant};
use crate::program::ui_system::UiConfig;
use imgui::FontConfig;

pub const UI_CONFIG: UiConfig = UiConfig {
    vsync: false,
    hardware_acceleration: Some(true),
};

pub const FONT_SIZES: [f32; 8] = [10f32, 12f32, 16f32, 24f32, 32f32, 40f32, 48f32, 64f32];
pub const DEFAULT_FONT_SIZE_INDEX: usize = 3;

pub const BUILTIN_FONTS: &[Font] = &[
    Font {
        //JB Mono has a no-ligatures version, but we like ligatures so ignore that one
        name: "JetBrains Mono",
        variants: &[
            FontVariant {
                name: "Thin",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-Thin.ttf"
                ),
            },
            FontVariant {
                name: "Extra Light",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-ExtraLight.ttf"
                ),
            },
            FontVariant {
                name: "Light",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-Light.ttf"
                ),
            },
            FontVariant {
                name: "Regular",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-Regular.ttf"
                ),
            },
            FontVariant {
                name: "Bold",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-Bold.ttf"
                ),
            },
            FontVariant {
                name: "Extra Bold",
                data: include_bytes!(
                    "../resources/fonts/JetBrains Mono v2.242/JetBrainsMono-ExtraBold.ttf"
                ),
            },
        ],
    },
    Font {
        name: "Consolas",
        variants: &[
            FontVariant {
                name: "Regular",
                data: include_bytes!("../resources/fonts/Consolas v5.53/Consolas.ttf"),
            },
            FontVariant {
                name: "Bold",
                data: include_bytes!("../resources/fonts/Consolas v5.53/Consolas Bold.ttf"),
            },
        ],
    },
    Font {
        name: "Fira Code",
        variants: &[
            FontVariant {
                name: "Regular",
                data: include_bytes!("../resources/fonts/Consolas v5.53/Consolas.ttf"),
            },
            FontVariant {
                name: "Bold",
                data: include_bytes!("../resources/fonts/Consolas v5.53/Consolas Bold.ttf"),
            },
        ],
    },
];
// Indices corresponding to the default font, in this case JB Mono @ Regular
pub const DEFAULT_FONT_INDEX: usize = 0;
pub const DEFAULT_FONT_VARIANT_INDEX: usize = 3;

// Fixed font size. Note imgui_winit_support uses "logical
// pixels", which are physical pixels scaled by the devices
// scaling factor. Meaning, 15.0 pixels should look the same size
// on two different screens, and thus we do not need to scale this
// value (as the scaling is handled by winit)
pub fn base_font_config() -> FontConfig {
    FontConfig {
        //TODO: Configure
        // Oversampling font helps improve text rendering at
        // expense of larger font atlas texture.
        oversample_h: 4,
        oversample_v: 4,
        ..FontConfig::default()
    }
}
