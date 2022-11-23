use super::engine;
use crate::build;
use crate::program::ui_system::{init_imgui_ui_system, UiConfig, UiSystem};
use color_eyre::eyre;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Surface};
use imgui::Ui;
use chrono::{Duration, Local};
use shadow_rs::DateTime;
use structx::*;
use tracing::{debug_span, info, instrument, span, trace, trace_span, Level, event};
use crate::helper::event_targets::*;
use crate::helper::logging;
mod ui_system;

#[derive(Debug, Copy, Clone)]
struct Program {
    pub demo_window_opened: bool,
}

///Creates and runs the program, returning once it has completed (probably when main window is closed)
pub(crate) fn run() -> eyre::Result<()> {
    let mut ui_system = init_imgui_ui_system(
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
    let mut instance = Program {
        demo_window_opened: false,
    };
    let mut last_frame = Local::now();

    let imgui_internal_span = debug_span!("imgui_internal");

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let guard_imgui_internal_span = imgui_internal_span.enter();
    ui_system
        .event_loop
        .run_return(move |event, _window_target, control_flow| {
            event!(target:UI_SPAMMY, Level::TRACE,"ui event: {event:?}");
            match event {
                //We have new events, but we don't care what they are, just need to update frame timings
                glutin::event::Event::NewEvents(_) => {
                    let old_last_frame = last_frame;
                    last_frame = Local::now();
                    let delta = last_frame - old_last_frame;
                    ui_system
                        .imgui_context
                        .io_mut()
                        .update_delta_time(delta.to_std().unwrap_or(std::time::Duration::from_secs(0)));

                    event!(target:UI_SPAMMY, Level::TRACE,"updated deltaT: old={}, new={}, delta={}", old_last_frame, last_frame, delta);
                }

                glutin::event::Event::MainEventsCleared => {
                    let gl_window = ui_system.display.gl_window();
                    ui_system
                        .platform
                        .prepare_frame(ui_system.imgui_context.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw(); //Pretty sure this makes us render constantly
                }

                //This only gets called when something changes (not constantly), but it doesn't matter too much since it should be real-time
                glutin::event::Event::RedrawRequested(_) => {
                    let ui = ui_system.imgui_context.frame();

                    //This is where we have to actually do the rendering
                    let render_span = trace_span!(target:UI_SPAMMY, parent: None, "render").entered();
                    render(&mut instance, &ui);
                    render_span.exit();

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

                //Catch-all, passes onto the glutin backend
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

    Ok(())
}

/// Called every frame, only place where rendering can occur
fn render(program: &mut Program, ui: &Ui) {
    ui.show_demo_window(&mut program.demo_window_opened);
    if ui.button("Panic (crash)") {
        panic!("Crashed by user manually clicking panic button");
    }
}
