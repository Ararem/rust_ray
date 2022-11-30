use crate::config::program_config::APP_TITLE;
use crate::helper::logging::event_targets::*;
use crate::program::ui_system::font_manager::FontManager;
use crate::program::ui_system::{init_ui_system, UiBackend, UiManagers, UiSystem};
use chrono::Local;
use color_eyre::{eyre, Report};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::{glutin, Display, Surface};
use imgui::{Condition, Context, DrawData, FontId, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::WinitPlatform;
use std::borrow::Borrow;
use std::ops::Deref;
use tracing::{debug_span, error, instrument, trace, trace_span, warn};
use crate::program::ui_system::docking::UiDocking;

pub(crate) mod ui_system;

/// Called every frame, only place where rendering can occur
fn render(
    display: &mut Display,
    imgui_context: &mut Context,
    platform: &mut WinitPlatform,
    renderer: &mut Renderer,
    managers: &mut UiManagers,
) -> color_eyre::Result<()> {
    let _guard = trace_span!(
        target: UI_SPAMMY,
        "render",
        frame = imgui_context.frame_count()
    )
        .entered();

    {
        let mut fonts = imgui_context.fonts();
        match managers.font_manager.rebuild_font_if_needed(&mut fonts) {
            Err(err) => warn!("font manager was not able to rebuild font: {err}"),
            // Font atlas was rebuilt
            Ok(true) => {
                drop(fonts);
                //Have to drop because it references imgui_context, and we need to borrow as mut here
                trace!("font was rebuilt, reloading renderer texture");
                let result = renderer.reload_font_texture(imgui_context);
                match result {
                    Ok(_) => trace!("renderer font texture reloaded successfully"),
                    Err(err) => {
                        let report = Report::new(err);
                        error!("{report}");
                    }
                }
            }
            Ok(false) => { trace!(target:UI_SPAMMY, "font not rebuilt (probably not dirty)") }
        }
    }

    // Create a new imgui frame to render to
    let ui = imgui_context.new_frame();
    //Build the UI
    {
        // Try to set our custom font
        let maybe_font_token = match managers.font_manager.get_font_id() {
            Err(err) => {
                warn!(
                target: UI_SPAMMY,
                "font manager failed to return font: {err}"
            );
                None
            }
            Ok(font_id) => Some(ui.push_font(*font_id)),
        };

        let flags =
            // No borders etc for top-level window
            imgui::WindowFlags::NO_DECORATION | imgui::WindowFlags::NO_MOVE
                // Show menu bar
                | imgui::WindowFlags::MENU_BAR
                // Don't raise window on focus (as it'll clobber floating windows)
                | imgui::WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | imgui::WindowFlags::NO_NAV_FOCUS
                // Don't want the dock area's parent to be dockable!
                | imgui::WindowFlags::NO_DOCKING
            ;

        // Remove padding/rounding on main container window
        let mw_style_tweaks = {
            let padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
            let rounding = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
            (padding, rounding)
        };

        // Create top-level window which occuplies full screen
        ui.window("Main Window")
          .flags(flags)
          .position([0.0, 0.0], imgui::Condition::Always)
          .size(ui.io().display_size, imgui::Condition::Always)
          .build(|| {

              // Create top-level docking area, needs to be made early (before docked windows)
              let ui_d = UiDocking {};
              let space = ui_d.dockspace("MainDockArea");

              // Set up splits, docking windows. This can be done conditionally,
              // or calling it every time is also mostly fine
              space.split(
                  imgui::Direction::Left,
                  0.7,
                  |left| {
                      left.dock_window("Window 1");
                  },
                  |right| {
                      // Further subdivide right-hand split
                      right.split(
                          imgui::Direction::Up,
                          0.5,
                          |up| {
                              up.dock_window("Window 2");
                          },
                          |down| {
                              down.dock_window("Window 3");
                          },
                      );
                  },
              );

              // Create application windows as normal
              ui.window("Window 1")
                .size([300.0, 110.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Window 1");
                });
              ui.window("Window 2")
                .size([300.0, 110.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Window 2");
                });
              ui.window("Window 3").build(|| {
                  ui.text("Window 3");
              });
          });

        ui.show_demo_window(&mut false);

        // Create stuff for our newly-created frame
        managers.render_ui_managers_window(&ui);

        if let Some(token) = maybe_font_token {
            token.pop();
        }
    }

    // Start drawing to our OpenGL context (via glium/glutin)
    let gl_window = display.gl_window();
    let mut target = display.draw();
    target.clear_color_srgb(0.0, 0.0, 0.0, 0.0); //Clear background so we don't get any leftovers from previous frames

    // Render our imgui frame now we've written to it
    platform.prepare_render(&ui, gl_window.window());
    let draw_data = imgui_context.render();

    // Copy the imgui rendered frame to our OpenGL surface (so we can see it)
    renderer
        .render(&mut target, draw_data)
        .expect("UI rendering failed");
    target.finish().expect("Failed to swap buffers");

    drop(_guard);
    return Ok(());
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

    let mut result = Ok(());

    let result_ref = &mut result;

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let imgui_internal_span = debug_span!("imgui_internal");
    let _guard_imgui_internal_span = imgui_internal_span.enter();
    event_loop.run_return(move |event, _window_target, control_flow| {
        let _span = trace_span!(target: UI_SPAMMY, "process_ui_event", ?event).entered();
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

                if let Err(e) = result {
                    let error = Report::wrap_err(e, "encountered error while rendering: {err}. program should now exit");
                    error!("{error}");
                    *result_ref = Err(error);
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

        _span.exit();
    });

    return result;
}
