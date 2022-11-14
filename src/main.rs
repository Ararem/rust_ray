#![warn(missing_docs)]

//! A little test raytracer project
mod boilerplate;
mod engine;
mod program;

use crate::boilerplate::error_handling::init_eyre;
use crate::boilerplate::logging::init_tracing;
use crate::boilerplate::ui_system::{init_imgui, UiConfig};
use color_eyre::eyre;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Surface};
use pretty_assertions::{self};
use shadow_rs::shadow;
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
fn main() -> eyre::Result<()> {
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
    let program = program::Program { test: true };
    let mut last_frame = Instant::now();

    let run_span = info_span!("run");
    let imgui_internal_span = debug_span!("imgui_internal");
    let _run_span_entered = run_span.enter();
    let _imgui_internal_span_entered = imgui_internal_span.enter();

    let exit_code:i32 = ui_system
        .event_loop
        .run_return(|event, _window_target, control_flow| {
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
                    // imgui_trace_span.exit();
                    program.tick(&ui);
                    // imgui_trace_span.enter();

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

    drop(_run_span_entered);

    info!("exit_code: {exit_code}");
    exit_code
}
