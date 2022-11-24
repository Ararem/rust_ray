use crate::config::program_config::APP_TITLE;
use crate::helper::logging::event_targets::*;
use crate::program::ui_system::init_ui_system;
use chrono::Local;
use color_eyre::eyre;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Surface};
use imgui::{FontId, Ui};
use tracing::{debug_span, error, instrument, trace, trace_span, warn};

pub(crate) mod ui_system;

#[derive(Debug, Copy, Clone)]
struct Program {
    pub demo_window_opened: bool,
    current_font: Option<FontId>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            demo_window_opened: false,
            current_font: None,
        }
    }
}

///Creates and runs the program, returning once it has completed (probably when main window is closed)
#[instrument(ret)]
pub(crate) fn run() -> eyre::Result<()> {
    let mut ui_system = init_ui_system(&APP_TITLE, crate::config::ui_config::UI_CONFIG)?;
    trace!("creating program instance");
    let mut instance = Program::new();
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
                    let ctx = &mut ui_system.imgui_context;
                    let ui = ctx.frame();

                    //This is where we have to actually do the rendering
                    let render_span =
                        trace_span!(target: UI_SPAMMY, "render", frame = ui.frame_count())
                            .entered();
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
    let font_token;
    if let Some(f) = program.current_font {
        font_token = Some(ui.push_font(f));
    } else {
        warn!("program.current_font was `None`, expected to have a font");
        font_token = None;
    }

    ui.show_demo_window(&mut program.demo_window_opened);
    font_manager::render_font_selector();

    if let Some(f) = font_token {
        f.pop();
    }
}
