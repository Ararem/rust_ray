#![warn(missing_docs)]

//! # A little test raytracer project
mod core;
mod build_config;
mod engine;
mod program;

use crate::core::error_handling::init_eyre;
use crate::core::logging::init_tracing;
use crate::core::ui_system::{init_imgui, UiConfig};
use color_eyre::eyre;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Surface};
use shadow_rs::shadow;
use std::process::ExitCode;
use std::time::Instant;
use tracing::*;

shadow!(build); //Required for shadow-rs to work

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises `eyre` (for panic/error handling)
/// * Initialises `tracing` (for logging)
/// * Processes command-line arguments
/// * Runs the program for real
fn main() -> eyre::Result<ExitCode> {
    init_eyre()?;
    init_tracing()?;
    debug!("[tracing] and [eyre] initialised");

    debug!("Skipping CLI and Env args");

    let mut ui_system = init_imgui(
        std::format!(
            "{} v{} - {}",
            build::PROJECT_NAME,
            build::PKG_VERSION,
            build::BUILD_TARGET
        )
        .as_str(),
        UiConfig {
            vsync: true,
            hardware_acceleration: Some(true),
        },
    )?;

    //Event loop
    debug!("init complete, starting");

    trace!("creating new program instance");
    let program = log_expr_val!(program::Program { test: true }, Debug);
    let add_two_numbers = log_expr!(f64::from(5+5) * 3.21f64, custom_expression_name, "Adding numbers: {custom_expression_name}");
    let mut last_frame = Instant::now();

    let run_span = info_span!("run");
    let imgui_internal_span = debug_span!("imgui_internal");

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let guard_imgui_internal_span = imgui_internal_span.enter();
    ui_system
        .event_loop
        .run_return(move |event, _window_target, control_flow| {
            if build_config::tracing::ENABLE_UI_TRACE {
                trace!("ui event: {event:?}");
            } //Log UI event if required
            match event {
                //We have new events, but we don't care what they are, just need to update frame timings
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

                //This only gets called when something changes (not constantly), but it doesn't matter too much since it should be real-time
                glutin::event::Event::RedrawRequested(_) => {
                    let ui = ui_system.imgui_context.frame();

                    //This is where we have to actually do the rendering
                    program.tick(&ui);

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

                //Handle window events, we just do close events
                glutin::event::Event::WindowEvent {
                    event: glutin::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                //Catch-all passes onto the glutin backend
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

    drop(guard_imgui_internal_span);

    info!("Goodbye");
    return Ok(ExitCode::SUCCESS);
}
