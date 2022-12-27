use std::sync::mpsc::TrySendError::{Disconnected, Full};
use std::sync::{Arc, Barrier, Mutex, TryLockError};
use std::thread::sleep;
use std::time::{Duration, Instant};

use color_eyre::eyre::WrapErr;
use color_eyre::{eyre, Help, Report};
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use glium::glutin::platform::windows::EventLoopBuilderExtWindows;
use glium::glutin::CreationError::NoAvailablePixelFormat;
use glium::{glutin, Display, Surface};
use imgui::Condition::Always;
use imgui::{Context, StyleVar, WindowFlags};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::event_loop::EventLoopBuilder;
use imgui_winit_support::winit::window::WindowBuilder;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::field::{debug, Empty};
use tracing::{debug, debug_span, error, info, info_span, trace, trace_span, warn};

use crate::build::*;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_error;
use crate::program::program_data::ProgramData;
use crate::program::thread_messages::ThreadMessage::{Engine, Program, Ui};
use crate::program::thread_messages::*;
use crate::ui::build_ui_impl::build_ui;
use crate::ui::docking::UiDockingArea;
use crate::ui::font_manager::FontManager;
use crate::ui::ui_data::UiData;
use crate::ui::ui_system::{FrameInfo, UiBackend, UiManagers, UiSystem};
use crate::FallibleFn;
use ProgramThreadMessage::QuitAppNoError;
use QuitAppNoErrorReason::QuitInteractionByUser;
use crate::config::Config;

mod build_ui_impl;
mod clipboard_integration;
mod docking;
mod font_manager;
pub mod ui_data;
mod ui_system;

pub(crate) fn ui_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<ThreadMessage>,
    message_receiver: BroadcastReceiver<ThreadMessage>,
    config: Config
) -> FallibleFn {
    let span_ui_thread =
        info_span!(target: THREAD_DEBUG_GENERAL, parent: None, "ui_thread").entered();

    {
        let span_sync_thread_start =
            debug_span!(target: THREAD_DEBUG_GENERAL, "sync_thread_start").entered();
        trace!(
            target: THREAD_DEBUG_GENERAL,
            "waiting for {}",
            name_of!(thread_start_barrier)
        );
        thread_start_barrier.wait();
        trace!(
            target: THREAD_DEBUG_GENERAL,
            "wait complete, running ui thread"
        );
        span_sync_thread_start.exit();
    }

    /*
    Init ui
    If we fail here, it is considered a fatal error (an so the thread exits), since I don't have any good way of fixing the errors
    */
    let system = init_ui_system(format!("{} v{} - {}", PROJECT_NAME, PKG_VERSION, BUILD_TARGET).as_str(), config).wrap_err("failed while initialising ui system")?;

    // Pulling out the separate variables is the only way I found to avoid getting "already borrowed" errors everywhere
    // Probably because I was borrowing the whole struct when I only needed one field of it
    let UiSystem {
        backend:
            UiBackend {
                mut display,
                mut event_loop,
                mut imgui_context,
                mut platform,
                mut renderer,
            },
        mut managers,
    } = system;

    /*
    Since we can't technically pass a variable out of a closure (which we have to use for the event loop),
    Let the event loop take ownership of `result_ref`, and use `result` afterwards.
    Neat!
    */
    let mut result: FallibleFn = Ok(());
    #[allow(unused_variables)]
    //It's not unused [event_loop_return()] macro uses it but it's not recognised
    let result_ref = &mut result;
    let mut last_frame = Instant::now();

    debug!(target: UI_DEBUG_GENERAL, "running event loop");
    let span_event_loop_internal =
        debug_span!(target: UI_DEBUG_GENERAL, "event_loop_internal").entered();
    event_loop.run_return(|event, _window_target, control_flow| {
        /// Macro that makes the event loop exit with a specified value
        macro_rules! event_loop_return {
            ($return_value:expr) => {{
                let ret = $return_value;
                trace!(target: UI_DEBUG_GENERAL, r#return=?ret, "expecting event loop to exit");
                *result_ref = ret;
                *control_flow = ControlFlow::Exit;
                return;
            }};
        }

        // Span for the entire closure that is called by [run_return]
        let span_process_ui_event_closure = trace_span!(target: UI_TRACE_EVENT_LOOP, "process_ui_event", ?event, ?control_flow).entered();
        match event {
            //We have new events, but we don't care what they are, just need to update frame timings
            glutin::event::Event::NewEvents(_) => {
                let old_last_frame = last_frame;
                last_frame = Instant::now();
                let delta = last_frame - old_last_frame;
                imgui_context.io_mut().update_delta_time(delta);

                trace!(
                    target: UI_TRACE_EVENT_LOOP,
                    "updated deltaT: {}",
                    humantime::format_duration(delta)
                );
            }

            glutin::event::Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                let window = gl_window.window();
                //Pretty sure this makes us render constantly since we always want the app to be drawing (realtime application remember)
                trace_span!(target: UI_TRACE_EVENT_LOOP, "request_redraw").in_scope(|| window.request_redraw());

                trace_span!(target: UI_TRACE_EVENT_LOOP, "prepare_frame").in_scope(|| {
                    let result = platform.prepare_frame(imgui_context.io_mut(), window);
                    if let Err(error) = result {
                        let report = Report::new(error)
                            .wrap_err("failed to prepare frame")
                            .note("this error probably isn't harmful and shouldn't break anything");
                        // Pretty sure this error isn't harmful, so just log it
                        warn!(target: GENERAL_WARNING_NON_FATAL, "failed to prepare frame: {}", report);
                    }
                });
            }

            glutin::event::Event::RedrawRequested(_) => {
                let span_redraw = trace_span!(target: UI_TRACE_EVENT_LOOP, "redraw").entered();

                let mut program_data = {
                    const MUTEX_LOCK_RETRY_DELAY: Duration = Duration::from_millis(1);

                    let span_obtain_data = trace_span!(target:THREAD_TRACE_MUTEX_SYNC, "obtain_data", ?MUTEX_LOCK_RETRY_DELAY, tries = Empty, time_taken_to_obtain = Empty).entered();

                    let mut tries = 0;
                    let start = Instant::now();
                    let program_data = loop {
                        tries += 1;
                        match program_data_wrapped.try_lock() {
                            //Shouldn't get here, since the engine/main threads shouldn't panic (and the app should quit if they do)
                            Err(TryLockError::Poisoned(_)) => {
                                let report = Report::msg("program data mutex poisoned").note("another thread panicked while holding the lock").suggestion("the error did not occur here (and has nothing to do with here), check the other threads and their logs").wrap_err("could not lock program data mutex").wrap_err("could not obtain program data");
                                error!(target: DOMINO_EFFECT_FAILURE, ?report);
                                event_loop_return!(Err(report));
                            }
                            Err(TryLockError::WouldBlock) => {
                                trace!(target: THREAD_TRACE_MUTEX_SYNC, "mutex locked, waiting and retrying");
                                sleep(MUTEX_LOCK_RETRY_DELAY);
                                continue;
                            }
                            Ok(data) => {
                                trace!(target: THREAD_TRACE_MUTEX_SYNC, ?data, "obtained program data");
                                break data;
                            }
                        }
                    };
                    span_obtain_data.record("tries", tries);
                    span_obtain_data.record("time_taken_to_obtain", tracing::field::debug(Instant::now() - start));
                    span_obtain_data.exit();

                    program_data
                };


                // Makes it easier to separate out frames
                // Add 1 to the frame count, since "technically" we're in the previous frame, as we haven't started the next one yet (call `new_frame()`)
                trace!(target: UI_TRACE_RENDER, "{0} BEGIN RENDER FRAME {frame} {0}", str::repeat("=", 50), frame = imgui_context.frame_count() + 1);

                let render_frame_result = outer_render_a_frame(
                    &mut display,
                    &mut imgui_context,
                    &mut platform,
                    &mut renderer,
                    &mut managers,
                    &mut program_data.ui_data,
                    &message_sender,
                    &message_receiver,
                     config
                );

                trace!(target: UI_TRACE_RENDER, "{0} END RENDER FRAME {frame} {0}", str::repeat("=", 50), frame = imgui_context.frame_count());


                if let Err(error) = render_frame_result {
                    let error = error.wrap_err("errored while rendering frame")
                                     .note("the program should exit");
                    error!(target: GENERAL_ERROR_FATAL, ?error);
                    event_loop_return!(Err(error));
                }

                span_redraw.exit();
            }

            //Handle window events, we just do close events
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                // Here, we don't actually want to close the window, but inform the main thread that we'd like to quit
                // Then, we wait for the main thread to tell us to quit
                let span_close_requested = debug_span!(target: UI_DEBUG_USER_INTERACTION, "close_requested").entered();

                let message = Program(QuitAppNoError(QuitInteractionByUser));
                debug_span!(target: THREAD_DEBUG_MESSAGE_SEND, "send_quit_signal", ?message).in_scope(|| {
                    match message_sender.try_send(message) {
                        Ok(()) => {
                            // We have signalled the thread, wait till the next loop when the main thread wants us to exit
                            debug!(target:THREAD_DEBUG_MESSAGE_SEND, "program thread signalled, should exit soon");
                            debug!(target: UI_DEBUG_GENERAL, "see you on the other side!");
                        }

                        // Neither of these errors should happen ever, but better to be safe
                        Err(Disconnected(_failed_message)) => {
                            event_loop_return!(Err(error_send_never_should_be_disconnected()));
                        }
                        Err(Full(_failed_message)) => {
                            event_loop_return!(Err(error_never_should_be_full()));
                        }
                    }
                });
                span_close_requested.exit();
            }

            //Catch-all, passes onto the glutin backend
            event => {
                let span_event_passthrough = trace_span!(target: UI_TRACE_EVENT_LOOP, "event_passthrough").entered();
                let gl_window = display.gl_window();
                platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
                span_event_passthrough.exit();
            }
        }

        if let Some(ret) = process_messages_with_return(&message_sender, &message_receiver) {
            event_loop_return!(ret);
        }

        span_process_ui_event_closure.exit();
    });
    span_event_loop_internal.exit();

    // If we get to here, it's time to exit the thread and shutdown
    info!(target: THREAD_DEBUG_GENERAL, "ui thread exiting");

    trace!(
        target: THREAD_DEBUG_MESSENGER_LIFETIME,
        "unsubscribing message receiver"
    );
    message_receiver.unsubscribe();
    trace!(
        target: THREAD_DEBUG_MESSENGER_LIFETIME,
        "unsubscribing message sender"
    );
    message_sender.unsubscribe();

    debug!(target: THREAD_DEBUG_GENERAL, "ui thread done");
    span_ui_thread.exit();
    Ok(())
}

/// Called every frame, handles everything required to draw an entire frame
///
/// Sets up all the boilerplate, then calls [inner_render] to create (build) the actual ui
#[allow(clippy::too_many_arguments)/*I literally can't join them into a type, or I can't borrow*/]
fn outer_render_a_frame(
    display: &mut Display,
    imgui_context: &mut Context,
    platform: &mut WinitPlatform,
    renderer: &mut Renderer,
    managers: &mut UiManagers,
    ui_data: &mut UiData,
    message_sender: &BroadcastSender<ThreadMessage>,
    message_receiver: &BroadcastReceiver<ThreadMessage>,
    config: Config
) -> FallibleFn {
    let span_outer_render = trace_span!(
        target: UI_TRACE_RENDER,
        "outer_render",
        frame = imgui_context.frame_count() + 1, // haven't called [new_frame()] yet, so the count hasn't incremented
        "time_to_render" = Empty
    )
    .entered();
    let start_outer_render = Instant::now();

    trace_span!(target: UI_TRACE_RENDER, "maybe_rebuild_font").in_scope(|| {
        let fonts = imgui_context.fonts();
        match managers.font_manager.rebuild_font_if_needed(fonts) {
            Err(report) => {
                let report = report.wrap_err("font manager was not able to rebuild font");
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    report = format_error(&report, config)
                );
            }
            // Font atlas was rebuilt
            Ok(true) => {
                trace!(
                    target: UI_TRACE_RENDER,
                    "font was rebuilt, reloading renderer texture"
                );
                let result = renderer.reload_font_texture(imgui_context);
                match result {
                    Ok(()) => trace!(
                        target: UI_TRACE_RENDER,
                        "renderer font texture reloaded successfully"
                    ),
                    Err(err) => {
                        let report =
                            Report::new(err).wrap_err("renderer could not reload font texture");
                        warn!(
                            target: GENERAL_WARNING_NON_FATAL,
                            report = format_error(&report, config)
                        );
                    }
                }
            }
            Ok(false) => {
                trace!(
                    target: UI_TRACE_RENDER,
                    "font not rebuilt (probably not dirty)"
                )
            }
        }
    });

    // Create a new imgui frame to render to
    trace!(target: UI_TRACE_RENDER, "new_frame()");
    let ui = imgui_context.new_frame();
    trace!(target: UI_TRACE_RENDER, new_frame=?ui);
    //Build the UI
    {
        let span_outer_build_ui =
            trace_span!(target: UI_TRACE_BUILD_INTERFACE, "outer_build_ui").entered();
        // Try to set our custom font
        let maybe_font_token = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "apply_custom_font")
            .in_scope(|| match managers.font_manager.get_font_id() {
                Err(report) => {
                    let report = report.wrap_err("font manager failed to return font");
                    warn!(
                        target: GENERAL_WARNING_NON_FATAL,
                        report = format_error(&report, config)
                    );
                    trace!(
                        target: UI_TRACE_BUILD_INTERFACE,
                        "could not get custom font"
                    );
                    None
                }
                Ok(font_id) => {
                    trace!(
                        target: UI_TRACE_BUILD_INTERFACE,
                        ?font_id,
                        "got custom font, applying"
                    );
                    Some(ui.push_font(*font_id))
                }
            });

        let main_window_flags =
            // No borders etc for top-level window
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                // Show menu bar
                | WindowFlags::MENU_BAR
                // Don't raise window on focus (as it'll clobber floating windows)
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | WindowFlags::NO_NAV_FOCUS
                // Don't want the dock area's parent to be dockable!
                | WindowFlags::NO_DOCKING;
        let main_window_padding = StyleVar::WindowPadding([0.0, 0.0]);
        let main_window_position = [0.0, 0.0];
        let main_window_size = ui.io().display_size;
        let main_window_name = "Main Window";

        trace!(
            target: UI_TRACE_BUILD_INTERFACE,
            ?main_window_padding,
            "push window padding"
        );
        let window_padding_token = ui.push_style_var(main_window_padding);
        trace!(
            target: UI_TRACE_BUILD_INTERFACE,
            ?main_window_name,
            ?main_window_flags,
            ?main_window_position,
            ?main_window_size,
            "begin main window"
        );
        let main_window_token = ui
            .window(main_window_name)
            .flags(main_window_flags)
            .position(main_window_position, Always) // These two make it always fill the whole screen
            .size(main_window_size, Always)
            .begin();
        trace!(target: UI_TRACE_BUILD_INTERFACE, "end window padding");
        window_padding_token.end();

        //TODO: Remove unneeded docking code
        trace!(target: UI_TRACE_BUILD_INTERFACE, "build docking area");
        let docking_area = UiDockingArea {};
        let _dock_node = docking_area.dockspace("Main Dock Area");

        build_ui(ui, managers, ui_data, message_sender, message_receiver, config)
            .wrap_err("building ui failed")?;

        // Technically we should only build the UI if [maybe_window_token] is [Some] ([None] means the window is hidden)
        // The window should never be hidden though, so this is a non-issue and we ignore it
        if let Some(token) = main_window_token {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "end main window");
            token.end();
        } else {
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                "ui main window is hidden (should always be visible)"
            );
        }

        if let Some(token) = maybe_font_token {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "pop custom font token");
            token.pop();
        } else {
            trace!(
                target: UI_TRACE_BUILD_INTERFACE,
                "no custom font token to pop"
            );
        }

        span_outer_build_ui.exit();
    }

    // Start drawing to our OpenGL context (via glium/glutin)
    {
        let span_draw_frame = trace_span!(target: UI_TRACE_RENDER, "draw_frame").entered();
        let gl_window = display.gl_window();
        trace!(
            target: UI_TRACE_RENDER,
            "start drawing frame to backbuffer: `display.draw()`"
        );
        let mut target = display.draw();
        trace!(
            target: UI_TRACE_RENDER,
            "clearing buffer: `target.clear_color_srgb()`"
        );
        target.clear_color_srgb(0.0, 0.0, 0.0, 0.0); //Clear background so we don't get any leftovers from previous frames

        // Render our imgui frame now we've written to it
        trace!(
            target: UI_TRACE_RENDER,
            "preparing platform for render: `platform.prepare_render()`"
        );
        platform.prepare_render(ui, gl_window.window());
        trace!(
            target: UI_TRACE_RENDER,
            "rendering imgui frame: `imgui_context.render()`"
        );
        let draw_data = imgui_context.render();

        trace!(target: UI_TRACE_RENDER, "gl render: `renderer.render()`");
        renderer
            .render(&mut target, draw_data)
            .wrap_err("could not render draw data")?;
        trace!(
            target: UI_TRACE_RENDER,
            "swapping buffers: `target.finish()`"
        );
        target.finish().wrap_err("failed to swap buffers")?;

        trace!(target: UI_TRACE_RENDER, "render complete");

        span_draw_frame.exit();
    }

    span_outer_render.record("time_to_render", debug(Instant::now() - start_outer_render));
    span_outer_render.exit();
    Ok(())
}

/// Function that processes the messages, and returns a value depending on what the UI thread should do
///
/// # Return Value
/// [None] - Do nothing
/// [Some<T>] - UI thread main function should return the value of type T (either [Err()] or [Ok()])
fn process_messages_with_return(
    _message_sender: &BroadcastSender<ThreadMessage>,
    message_receiver: &BroadcastReceiver<ThreadMessage>,
) -> Option<FallibleFn> {
    let span_process_messages = trace_span!(
        target: THREAD_TRACE_MESSAGE_LOOP,
        name_of!(process_messages_with_return)
    )
    .entered();
    // Loops until [message_receiver] is empty (tries to 'flush' out all messages)
    'process_messages: loop {
        match receive_message(message_receiver) {
            // Couldn't receive, ***~~BAD~~***, UI Thread should return with the receive error
            Err(recv_error) => return Some(Err(recv_error)),
            //No messages waiting
            Ok(None) => break 'process_messages,
            Ok(Some(message)) => {
                match message {
                    Program(_) | Engine(_) => {
                        message.ignore();
                        continue 'process_messages;
                    }
                    Ui(ui_message) => {
                        debug!(
                            target: THREAD_DEBUG_MESSAGE_RECEIVED,
                            ?ui_message,
                            "got ui message"
                        );
                        return match ui_message {
                            UiThreadMessage::ExitUiThread => {
                                debug!(
                                    target: THREAD_DEBUG_GENERAL,
                                    "got exit message for Ui thread"
                                );
                                Some(Ok(())) //Ui thread should return with Ok
                            }
                        };
                    }
                }
            }
        }
    }

    //UI thread should keep running
    span_process_messages.exit();
    None
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
fn init_ui_system(title: &str, config: Config) -> eyre::Result<UiSystem> {
    let span_init_ui = debug_span!(target: UI_DEBUG_GENERAL, "init_ui").entered();

    let mut imgui_context;
    let event_loop;
    let mut platform;
    let renderer;

    //TODO: More config options
    debug!(target: UI_DEBUG_GENERAL, "cloning title");
    let title = title.to_owned();
    debug!(target: UI_DEBUG_GENERAL, title);

    debug!(
        target: UI_DEBUG_GENERAL,
        "creating [winit] event loop with [any_thread]=`true`"
    );
    event_loop = EventLoopBuilder::with_any_thread(&mut EventLoopBuilder::new(), true).build();
    debug!(
        target: UI_DEBUG_GENERAL,
        ?event_loop,
        "[winit] event loop created"
    );

    debug!(
        target: UI_DEBUG_GENERAL,
        "creating [glutin] context builder"
    );
    let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
        .with_vsync(config.init.ui_config.vsync)
        .with_hardware_acceleration(config.init.ui_config.hardware_acceleration)
        .with_srgb(true)
        .with_multisampling(config.init.ui_config.multisampling);
    debug!(
        target: UI_DEBUG_GENERAL,
        ?glutin_context_builder,
        "created [glutin] context builder"
    );

    debug!(target: UI_DEBUG_GENERAL, "creating [winit] window builder");
    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(config.init.ui_config.default_window_size)
        .with_maximized(config.init.ui_config.start_maximised);
    debug!(
        target: UI_DEBUG_GENERAL,
        ?window_builder,
        "created [winit] window builder"
    );
    //TODO: Configure
    debug!(target: UI_DEBUG_GENERAL, "creating [glium] display");
    let gl_display: Display = Display::new(window_builder, glutin_context_builder, &event_loop)
        .wrap_err("could not initialise display")
        .note(format!("if the error is [NoAvailablePixelFormat] (`{}`), try checking the [glutin::ContextBuilder] settings: vsync, hardware acceleration and srgb may not be a compatible combination on your system", NoAvailablePixelFormat))?;
    debug!(target: UI_DEBUG_GENERAL, display=?gl_display, "created [glium] display");

    debug!(target: UI_DEBUG_GENERAL, "creating [imgui] context");
    imgui_context = Context::create();
    debug!(
        target: UI_DEBUG_GENERAL,
        ?imgui_context,
        "created [imgui] context"
    );

    debug!(
        target: UI_DEBUG_GENERAL,
        "setting DOCKING_ENABLE flag for [imgui]"
    );
    imgui_context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
    debug!(target: UI_DEBUG_GENERAL, config_flags=?imgui_context.io().config_flags);

    let font_manager =
        debug_span!(target: UI_DEBUG_GENERAL, "create_font_manager").in_scope(|| {
            let mut font_manager = FontManager::new().wrap_err("failed to create font manager")?;
            debug!(target: UI_DEBUG_GENERAL, "loading font manager fonts list"); //Need to call it now or else we don't have any fonts loaded and the manager craps itself later
            font_manager.reload_list_from_resources()?;
            debug!(
                target: UI_DEBUG_GENERAL,
                ?font_manager,
                "created font manager"
            );
            eyre::Result::<FontManager>::Ok(font_manager)
        })?;

    //TODO: High DPI setting
    debug!(target: UI_DEBUG_GENERAL, "creating [winit] platform");
    platform = WinitPlatform::init(&mut imgui_context);
    debug!(
        target: UI_DEBUG_GENERAL,
        ?platform,
        "created [winit] platform"
    );

    debug!(target: UI_DEBUG_GENERAL, "attaching window to platform");
    platform.attach_window(
        imgui_context.io_mut(),
        gl_display.gl_window().window(),
        HiDpiMode::Default,
    );
    debug!(target: UI_DEBUG_GENERAL, "attached window to platform");

    debug!(target: UI_DEBUG_GENERAL, "creating [glium] renderer");
    renderer =
        Renderer::init(&mut imgui_context, &gl_display).wrap_err("failed to create renderer")?;
    debug!(target: UI_DEBUG_GENERAL, "created [glium] renderer");

    debug_span!(target: UI_DEBUG_GENERAL, "clipboard_init").in_scope(|| {
        match clipboard_integration::clipboard_init() {
            Ok(clipboard_backend) => {
                debug!(
                    target: UI_DEBUG_GENERAL,
                    ?clipboard_backend,
                    "have clipboard support"
                );
                imgui_context.set_clipboard_backend(clipboard_backend);
                debug!(target: UI_DEBUG_GENERAL, "clipboard backend set");
            }
            Err(report) => {
                let report = report.wrap_err("could not initialise clipboard");
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    report = format_error(&report, config),
                    "could not init clipboard"
                );
            }
        }
    });

    debug!(target: UI_DEBUG_GENERAL, "ui init done");
    span_init_ui.exit();
    Ok(UiSystem {
        backend: UiBackend {
            event_loop,
            display: gl_display,
            imgui_context,
            platform,
            renderer,
        },
        managers: UiManagers {
            font_manager,
            frame_info: FrameInfo::new(),
        },
    })
}
