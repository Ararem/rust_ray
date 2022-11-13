use color_eyre::eyre::WrapErr;
use color_eyre::{eyre, Report};
use glium::backend::glutin::DisplayCreationError;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::{Window, WindowBuilder};
use glium::{glutin, Display};
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use tracing::{debug, debug_span, instrument, trace, trace_span, warn, Instrument};

mod clipboard;

///Struct that encapsulates the UI system components
pub struct UiSystem {
    pub display: glium::Display,
    pub event_loop: EventLoop<()>,
    pub imgui_context: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
}

#[derive(Debug)]
pub struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
    /// Optional multisampling
    pub multisampling: u16,
}

impl UiConfig {
    pub fn new() -> UiConfig {
        UiConfig {
            hardware_acceleration: None,
            vsync: true,
            multisampling: 1,
        }
    }
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
/// * `config` - Struct that modifies how the ui system is initialised
#[instrument]
pub fn init(title: &str, config: UiConfig) -> eyre::Result<UiSystem> {
    let display;
    let mut imgui;
    let event_loop;

    {
        let log_span = debug_span!("creating basic objects").entered();
        //TODO: More config options
        trace!("cloning title");
        let title = title.to_owned();
        trace!("creating event loop");
        event_loop = EventLoop::new();
        trace!("creating glutin context builder");
        let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
            .with_vsync(config.vsync)
            .with_hardware_acceleration(config.hardware_acceleration)
            .with_multisampling(config.multisampling);
        trace!("creating window builder");
        let window_builder = WindowBuilder::new().with_title(title); //TODO: Configure
        trace!("creating display");
        display = Display::new(window_builder, glutin_context_builder, &event_loop)
            .wrap_err("could not initialise display")?;
        trace!("Creating [imgui] context");
        imgui = Context::create();
    }

    {
        let log_span = debug_span!("trying to enable clipboard support").entered();
        match clipboard::init() {
            Ok(clipboard_backend) => {
                trace!("have clipboard support");
                imgui.set_clipboard_backend(clipboard_backend);
            }
            Err(error) => {
                warn!("could not initialise clipboard: {error}")
            }
        }
    }

    let mut platform;
    {
        let log_span = debug_span!("initialising winit platform").entered();
        platform = WinitPlatform::init(&mut imgui);
        let gl_window = display.gl_window();
        let window = gl_window.window();
        //TODO: High DPI setting
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
    }

    //TODO: Proper resource manager
    {
        let log_span = debug_span!("adding fonts").entered();

        // Fixed font size. Note imgui_winit_support uses "logical
        // pixels", which are physical pixels scaled by the devices
        // scaling factor. Meaning, 13.0 pixels should look the same size
        // on two different screens, and thus we do not need to scale this
        // value (as the scaling is handled by winit)
        let font_size = 13.0;
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
            rasterizer_multiply: 1.5,
            //Sets everything to default
            //Except the stuff we overrode before
            //SO COOOL!!
            ..FontConfig::default()
        };

        let fallback_font = FontSource::DefaultFontData {
            config: Some(font_config.clone()),
        };
        let standard_font = FontSource::TtfData {
            config: Some(font_config),
            size_pixels: font_size,
            data: include_bytes!(
                "../../resources/fonts/JetBrainsMono-2.242/fonts/ttf/JetBrainsMono-Medium.ttf"
            ),
        };

        trace!("standard font: {standard_font:?}, fallback: {fallback_font:?}");
        let fonts = &[standard_font, fallback_font];
        imgui.fonts().add_font(fonts);
    }

    debug!("creating renderer");
    let renderer = Renderer::init(&mut imgui, &display).wrap_err("failed to create renderer")?;

    Ok(UiSystem {
        event_loop,
        display,
        imgui_context: imgui,
        platform,
        renderer,
    })
}
