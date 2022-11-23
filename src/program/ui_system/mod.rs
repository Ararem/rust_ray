mod clipboard_integration;
mod font_manager;

use crate::log_expr;
use color_eyre::eyre;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use glium::{glutin, Display};
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use tracing::{debug, debug_span, instrument, trace, warn};

/*
TODO:   Add support for different renderers (glow, glium, maybe d3d12, dx11, wgpu) and backend platforms (winit, sdl2)
        Should probably add an enum selection to [UiConfig]. Also see if it's possible to change hotly or requires a reload
*/

/// Struct that encapsulates the UI system components
pub struct UiSystem {
    pub display: Display,
    pub event_loop: EventLoop<()>,
    pub imgui_context: Context,
    pub platform: WinitPlatform,
    /// The renderer that renders the current UI system
    pub renderer: Renderer,
}

/// Struct used to configure the UI system
#[derive(Debug, Copy, Clone)]
pub struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
/// * `config` - Struct that modifies how the ui system is initialised
#[instrument]
pub fn init_ui_system(title: &str, config: UiConfig) -> eyre::Result<UiSystem> {
    let display;
    let mut imgui;
    let event_loop;
    let mut platform;
    let renderer;

    {
        debug_span!("window_internals");
        //TODO: More config options
        trace!("cloning title");
        let title = title.to_owned();
        trace!("creating [winit] event loop");
        event_loop = EventLoop::new();
        trace!("creating [glutin] context builder");
        let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
            .with_vsync(config.vsync)
            .with_hardware_acceleration(config.hardware_acceleration);
        trace!("creating [winit] window builder");
        let window_builder = WindowBuilder::new().with_title(title); //TODO: Configure
        trace!("creating display");
        display = Display::new(window_builder, glutin_context_builder, &event_loop)
            .expect("could not initialise display");
        trace!("Creating [imgui] context");
        imgui = Context::create();
        // imgui.set_ini_filename(Some(PathBuf::from("./imgui.ini")));
        // imgui.set_log_filename()
        //TODO: High DPI setting
        trace!("creating [winit] platform");
        platform = WinitPlatform::init(&mut imgui);
        trace!("attaching window");
        platform.attach_window(
            imgui.io_mut(),
            display.gl_window().window(),
            HiDpiMode::Default,
        );

        debug!("creating [glium] renderer");
        renderer = Renderer::init(&mut imgui, &display).expect("failed to create renderer");
    }

    {
        debug_span!("clipboard_init");
        match clipboard_integration::init() {
            Ok(clipboard_backend) => {
                trace!("have clipboard support: {clipboard_backend:?}");
                imgui.set_clipboard_backend(clipboard_backend);
            }
            Err(error) => {
                warn!("could not initialise clipboard: {error}")
            }
        }
    }

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


    Ok(UiSystem {
        event_loop,
        display,
        imgui_context: imgui,
        platform,
        renderer,
    })
}
