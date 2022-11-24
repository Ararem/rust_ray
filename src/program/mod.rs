use crate::build;
use crate::helper::logging::event_targets::*;
use crate::program::ui_system::{init_ui_system, UiConfig};
use chrono::Local;
use color_eyre::eyre;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, program, Surface};
use imgui::Ui;
use tracing::{debug_span, event, instrument, span, trace, trace_span, Level};
pub(crate) mod ui_system;

#[derive(Debug, Copy, Clone)]
struct Program {
    pub demo_window_opened: bool,
}

///Creates and runs the program, returning once it has completed (probably when main window is closed)
#[instrument(ret)]
pub(crate) fn run() -> eyre::Result<()> {
    let mut ui_system = init_ui_system(
        std::format!(
            "{} v{} - {}",
            build::PROJECT_NAME,
            build::PKG_VERSION,
            build::BUILD_TARGET
        )
        .as_str(),
        crate::config::ui_config::UI_CONFIG,
    )?;
    trace!("creating program instance");
    let mut instance = Program {
        demo_window_opened: false,
    };
    trace!(?instance, "program instance created");
    let mut last_frame = Local::now();

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let imgui_internal_span = debug_span!("imgui_internal");
    let _guard_imgui_internal_span = imgui_internal_span.enter();
    ui_system
        .event_loop
        .run_return(move |event, _window_target, control_flow| {
            let _ = trace_span!(target: UI_SPAMMY, "process_ui_event", ?event).entered();
            match event {
                //We have new events, but we don't care what they are, just need to update frame timings
                glutin::event::Event::NewEvents(_) => {
                    let old_last_frame = last_frame;
                    last_frame = Local::now();
                    let delta = last_frame - old_last_frame;
                    ui_system.imgui_context.io_mut().update_delta_time(
                        delta.to_std().unwrap_or(std::time::Duration::from_secs(0)),
                    );

                    trace!(
                        target: UI_SPAMMY,
                        "updated deltaT: old={old_last_frame}, new={last_frame}, delta={delta}"
                    );
                }

                glutin::event::Event::MainEventsCleared => {
                    trace!(target: UI_SPAMMY, "requesting redraw");
                    let gl_window = ui_system.display.gl_window();
                    ui_system
                        .platform
                        .prepare_frame(ui_system.imgui_context.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw(); //Pretty sure this makes us render constantly
                }

                //This only gets called when something changes (not constantly), but it doesn't matter too much since it should be real-time
                glutin::event::Event::RedrawRequested(_) => {
                    trace!(target: UI_SPAMMY, "redraw requested");
                    let mut ctx = &mut ui_system.imgui_context;
                    let ui = ctx.frame();

                    //This is where we have to actually do the rendering
                    let render_span =
                        trace_span!(target: UI_SPAMMY, "render", frame = ui.frame_count()).entered();
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
