use std::borrow::Borrow;
use std::ops::Deref;
use crate::config::program_config::APP_TITLE;
use crate::helper::logging::event_targets::*;
use crate::program::ui_system::font_manager::FontManager;
use crate::program::ui_system::{init_ui_system, UiBackend, UiManagers, UiSystem};
use chrono::Local;
use color_eyre::{eyre, Report};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Display, Surface};
use imgui::{Context, DrawData, FontId, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::WinitPlatform;
use tracing::{debug_span, error, instrument, trace, trace_span, warn};

pub(crate) mod ui_system;

/// Called every frame, only place where rendering can occur
fn render(
    display: &mut Display,
    imgui_context: &mut Context,
    platform: &mut WinitPlatform,
    renderer: &mut Renderer,
    managers: &mut UiManagers,
) -> color_eyre::Result<()>{
    //Graphics stuff
    let _ = trace_span!(
        target: UI_SPAMMY,
        "render",
        frame = imgui_context.frame_count()
    )
    .entered();

    managers
        .font_manager
        .update_font(&mut imgui_context.fonts());

    // Create a new imgui frame to render to
    let ui = imgui_context.frame();

    // Create stuff for our newly-created frame
    managers.render_ui_window(&ui);

    // Start drawing to our OpenGL context (via glium/glutin)
    let gl_window = display.gl_window();
    let mut target = display.draw();
    target.clear_color_srgb(0.0, 0.0, 0.0, 0.0); //Clear background so we don't get any leftovers from previous frames

    // Render our imgui frame now we've written to it
    platform.prepare_render(&ui, gl_window.window());
    let draw_data = ui.render();

    // Copy the imgui rendered frame to our OpenGL surface (so we can see it)
    renderer
        .render(&mut target, draw_data)
        .expect("UI rendering failed");
    target.finish().expect("Failed to swap buffers");

    Ok(())
}

///Creates and runs the program, returning once it has completed (probably when main window is closed)
#[instrument(ret)]
pub(crate) fn run() -> eyre::Result<()> {
    let ui_system = init_ui_system(&APP_TITLE, crate::config::ui_config::UI_CONFIG)?;
    // Pulling out the separate variables is the only way I found to avoid getting "already borrowed" errors everywhere
    let UiSystem {
        backend,
        mut managers,
    } = ui_system;
    let UiBackend {
        mut display,
        mut event_loop,
        mut imgui_context,
        mut platform,
        mut renderer,
    } = backend;
    let mut last_frame = Local::now();

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let imgui_internal_span = debug_span!("imgui_internal");
    let _guard_imgui_internal_span = imgui_internal_span.enter();
    event_loop.run_return(move |event, _window_target, control_flow| {
        let _ = trace_span!(target: UI_SPAMMY, "process_ui_event", ?event).entered();
        match event {
            //We have new events, but we don't care what they are, just need to update frame timings
            glutin::event::Event::NewEvents(_) => {
                let old_last_frame = last_frame;
                last_frame = Local::now();
                let delta = last_frame - old_last_frame;
                imgui_context
                    .io_mut()
                    .update_delta_time(delta.to_std().unwrap_or(std::time::Duration::from_secs(0)));

                trace!(
                    target: UI_SPAMMY,
                    "updated deltaT: old={old_last_frame}, new={last_frame}, delta={delta}"
                );
            }

            glutin::event::Event::MainEventsCleared => {
                trace!(target: UI_SPAMMY, "requesting redraw");
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui_context.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw(); //Pretty sure this makes us render constantly
            }

            //This only gets called when something changes (not constantly), but it doesn't matter too much since it should be real-time
            glutin::event::Event::RedrawRequested(_) => {
                trace!(target: UI_SPAMMY, "redraw requested");

                let result = render(
                    &mut display,
                    &mut imgui_context,
                    &mut platform,
                    &mut renderer,
                    &mut managers,
                );

                if let Err(err) = result{
                    error!("encountered error while rendering: {err}. program should now exit");
                    *control_flow = ControlFlow::Exit;
                }
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
                let gl_window = display.gl_window();
                platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
            }
        }
    });

    return Ok(());
}
