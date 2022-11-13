#![warn(missing_docs)]

//! A little test raytracer project
mod core;
use crate::core::clipboard_integration;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::window::WindowBuilder;
use glium::{glutin, Display, Surface};
use imgui::{Context, FontConfig, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use std::io;
use std::time::Instant;
use tracing::metadata::LevelFilter;
use tracing::*;
use tracing_subscriber::{
    fmt::{format::*, time},
    util::TryInitError,
};

shadow!(build); //Required for shadow-rs to work

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
#[derive(Debug)]
pub struct UiConfig {
    pub vsync: bool,
    pub hardware_acceleration: Option<bool>,
}

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises `eyre` (for panic/error handling)
/// * Initialises `tracing` (for logging)
/// * Processes command-line arguments
/// * Runs the program for real
fn main() -> eyre::Result<()> {
    init_eyre()?;
    init_tracing()?;
    debug!("[tracing] and [eyre] initialised");

    debug!("Skipping CLI and Env args");

    let mut ui_system = init_imgui(
        "Test Title for the <APP>",
        UiConfig {
            vsync: true,
            hardware_acceleration: Some(true),
        },
    )?;

    //Event loop
    info_span!("run").in_scope(|| {
        let mut last_frame = Instant::now();
        ui_system.event_loop.run(move |event, _, control_flow| {
            // trace!("[ui] event: {event:?}");
            match event {
                glutin::event::Event::NewEvents(_) => {
                    ui_system
                        .imgui_context
                        .io_mut()
                        .update_delta_time(last_frame.elapsed());
                    last_frame = Instant::now();
                }

                glutin::event::Event::MainEventsCleared => {
                    let gl_window = ui_system.display.gl_window();
                    ui_system
                        .platform
                        .prepare_frame(ui_system.imgui_context.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw();
                }

                glutin::event::Event::RedrawRequested(_) => {
                    let ui = ui_system.imgui_context.frame();

                    //This is where we have to actually do the rendering
                    core::program::tick(&ui);

                    let gl_window = ui_system.display.gl_window();
                    let mut target = ui_system.display.draw();
                    target.clear_color_srgb(0.0, 0.0, 0.0, 0.0); //Clear
                    ui_system.platform.prepare_render(&ui, gl_window.window());
                    let draw_data = ui.render();
                    ui_system
                        .renderer
                        .render(&mut target, draw_data)
                        .expect("UI rendering failed");
                    target.finish().expect("Failed to swap buffers");
                }

                glutin::event::Event::WindowEvent {
                    event: glutin::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                event => {
                    let gl_window = ui_system.display.gl_window();
                    ui_system.platform.handle_event(
                        ui_system.imgui_context.io_mut(),
                        gl_window.window(),
                        &event,
                    );
                }
            }
        });
    })
}

fn init_tracing() -> eyre::Result<()> {
    use tracing_error::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    let standard_format = format()
        .compact()
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_level(true)
        .with_timer(time::time())
        .with_source_location(false)
        .with_level(true);

    let standard_layer = fmt::layer()
        .with_span_events(FmtSpan::ACTIVE)
        .log_internal_errors(true)
        .event_format(standard_format)
        .with_writer(io::stdout)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy()
        )
        // .with_test_writer()
        // .with_timer(time())
        ;

    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(standard_layer)
        .with(error_layer)
        .try_init()?;

    Ok(())
}

fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
/// * `config` - Struct that modifies how the ui system is initialised
#[instrument]
pub fn init_imgui(title: &str, config: UiConfig) -> eyre::Result<UiSystem> {
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
            .with_hardware_acceleration(config.hardware_acceleration);
        trace!("creating window builder");
        let window_builder = WindowBuilder::new().with_title(title); //TODO: Configure
        trace!("creating display");
        display = Display::new(window_builder, glutin_context_builder, &event_loop)
            .expect("could not initialise display");
        trace!("Creating [imgui] context");
        imgui = Context::create();
    }

    {
        let log_span = debug_span!("trying to enable clipboard support").entered();
        match clipboard_integration::init() {
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
                "resources/fonts/JetBrainsMono-2.242/fonts/ttf/JetBrainsMono-Medium.ttf"
            ),
        };

        let fonts = &[standard_font, fallback_font];
        imgui.fonts().add_font(fonts);
        trace!("added fonts");
    }

    debug!("creating renderer");
    let renderer = Renderer::init(&mut imgui, &display).expect("failed to create renderer");

    Ok(UiSystem {
        event_loop,
        display,
        imgui_context: imgui,
        platform,
        renderer,
    })
}
