use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use color_eyre::{eyre, Help, Report};
use color_eyre::eyre::WrapErr;
use glium::{Display, glutin};
use glium::glutin::CreationError::NoAvailablePixelFormat;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use imgui::Context;
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use imgui_winit_support::winit::event_loop::EventLoopBuilder;
use imgui_winit_support::winit::window::WindowBuilder;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug, debug_span, info, instrument, trace, trace_span, warn};

use crate::{log_expr_val, log_variable};
use crate::config::program_config::{APP_TITLE, IMGUI_LOG_FILE_PATH, IMGUI_SETTINGS_FILE_PATH};
use crate::config::ui_config::{DEFAULT_WINDOW_SIZE, HARDWARE_ACCELERATION, MULTISAMPLING, START_MAXIMIZED, VSYNC};
use crate::helper::logging::event_targets::*;
use crate::program::program_messages::{Message, UiThreadMessage, unreachable_never_should_be_disconnected};
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::ProgramData;
use crate::ui::ui_system::{UiBackend, UiManagers, UiSystem};

mod ui_system;
mod clipboard_integration;

#[derive(Copy, Clone, Debug)]
pub struct UiData {}

#[instrument(skip_all)]
pub(crate) fn ui_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<Message>,
    message_receiver: BroadcastReceiver<Message>,
) -> eyre::Result<()> {
    //Create a NoPanicPill to make sure we exit if anything panics
    let _no_panic_pill = crate::helper::panic_pill::PanicPill {};

    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running ui thread");

    /*
    Init ui
    If we fail here, it is considered a fatal error (an so the thread exits), since I don't have any good way of fixing the errors
    */
    let system = init_ui_system(APP_TITLE)
        .wrap_err("failed while initialising ui system")?;

    // Pulling out the separate variables is the only way I found to avoid getting "already borrowed" errors everywhere
    let UiSystem {
        backend: UiBackend {
            mut display,
            mut event_loop,
            mut imgui_context,
            mut platform,
            mut renderer,
        },
        managers: UiManagers {}
    };

    /*
    Since we can't technically pass a variable out of a closure (which we have to use for the event loop),
    Let the event loop take ownership of `result_ref`, and use `result` afterwards.
    Neat!
    */
    let mut result: eyre::Result<()> = Ok(());
    let result_ref = &mut result;
    let mut last_frame = Instant::now();

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let imgui_internal_span = debug_span!("imgui_internal");
    let _guard_imgui_internal_span = imgui_internal_span.enter();

    debug!("running event loop");
    event_loop.run_return(move |event, _window_target, control_flow| {
        let _span = trace_span!(target: UI_PERFRAME_SPAMMY, "process_ui_event", ?event).entered();
        match event {
            //We have new events, but we don't care what they are, just need to update frame timings
            glutin::event::Event::NewEvents(_) => {
                let old_last_frame = last_frame;
                last_frame = Instant::now();
                let delta = last_frame - old_last_frame;
                imgui_context
                    .io_mut()
                    .update_delta_time(delta);

                trace!(target: UI_PERFRAME_SPAMMY,"updated deltaT: {delta}",delta = humantime::format_duration(delta));
            }

            glutin::event::Event::MainEventsCleared => {
                trace!(target: UI_PERFRAME_SPAMMY, "main events cleared");
                trace!(target: UI_PERFRAME_SPAMMY, "requesting redraw");
                let window = display.gl_window().window();
                window.request_redraw();
                //Pretty sure this makes us render constantly
                trace!(target: UI_PERFRAME_SPAMMY, "preparing frame");
                let result = platform
                    .prepare_frame(imgui_context.io_mut(), window);
                if let Err(error) = result {
                    let report = Report::new(error);
                    // let wrapped_report = Report::wrap_err(report, "failed to prepare frame");
                    // Pretty sure this error isn't harmful, so just log it
                    warn!("failed to prepare frame: {}", report);
                }
            }

            //This only gets called when something changes (not constantly), but it doesn't matter too much since it should be real-time
            glutin::event::Event::RedrawRequested(_) => {
                trace!(target: UI_PERFRAME_SPAMMY, "redraw requested");

                let result = render(
                    &mut display,
                    &mut imgui_context,
                    &mut platform,
                    &mut renderer,
                    &mut managers,
                );

                if let Err(e) = result {
                    let error = Report::wrap_err(e, "encountered error while rendering: {err}. program should now exit");
                    warn!("encountered error while rendering");
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

    if let Some(ret) = process_messages(&message_sender, &message_receiver) {
        return ret;
    }

    // If we get to here, it's time to exit the thread and shutdown
    info!("ui thread exiting");

    trace!("unsubscribing message receiver");
    message_receiver.unsubscribe();
    trace!("unsubscribing message sender");
    message_sender.unsubscribe();

    trace!("dropping {}", name_of!(_no_panic_pill));
    drop(_no_panic_pill);
    return Ok(());
}

/// Function that processes the messages, and returns a value depending on what the UI thread should do
///
/// # Return Value
/// [None] - Do nothing
/// [Some<T>] - UI thread main function should return the value of type T (either [Err()] or [Ok()])
#[instrument(ret, skip_all, level = "trace")]
fn process_messages(message_sender: &BroadcastSender<Message>, message_receiver: &BroadcastReceiver<Message>) -> Option<eyre::Result<()>> {
    'loop_messages: loop {
        // Loops until [message_receiver] is empty (tries to 'flush' out all messages)
        let recv = message_receiver.try_recv();
        match recv {
            Err(TryRecvError::Empty) => {
                trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "no messages waiting");
                break 'loop_messages; // Exit the message loop, go into waiting
            }
            Err(TryRecvError::Disconnected) => {
                unreachable_never_should_be_disconnected();
            }
            Ok(message) => {
                trace!(
                        target: PROGRAM_MESSAGE_POLL_SPAMMY,
                        "got message: {:?}",
                        &message
                    );
                match message {
                    Program(_program_message) => {
                        trace!(
                                target: PROGRAM_MESSAGE_POLL_SPAMMY,
                                "[ui] message for program thread, ignoring"
                            );
                        continue 'loop_messages;
                    }
                    Engine(_engine_message) => {
                        trace!(
                                target: PROGRAM_MESSAGE_POLL_SPAMMY,
                                "[ui] message for engine thread, ignoring"
                            );
                        continue 'loop_messages;
                    }
                    Ui(ui_message) => match ui_message {
                        UiThreadMessage::ExitUiThread => {
                            trace!("got exit message for Ui thread");
                            return Some(Ok(())); //Ui thread should return with Ok
                        },
                    },
                }
            }
        }
    }

    //UI thread should keep running
    return None;
}

///Initialises the UI system and returns it
///
/// * `title` - Title of the created window
#[instrument]
fn init_ui_system(title: &str) -> eyre::Result<UiSystem> {
    let display;
    let mut imgui_context;
    let event_loop;
    let mut platform;
    let renderer;

    //TODO: More config options
    trace!("cloning title");
    let title = title.to_owned();
    log_variable!(title);

    trace!("creating [winit] event loop with [any_thread]=`true`");
    event_loop = EventLoopBuilder::new()
        .with_any_thread(true)
        .build();

    trace!("creating [glutin] context builder");
    let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
        .with_vsync(VSYNC)
        .with_hardware_acceleration(HARDWARE_ACCELERATION)
        .with_srgb(true)
        .with_multisampling(MULTISAMPLING);
    log_variable!(glutin_context_builder:?);

    trace!("creating [winit] window builder");
    let window_builder = WindowBuilder::new().with_title(title).with_inner_size(DEFAULT_WINDOW_SIZE).with_maximized(START_MAXIMIZED);
    //TODO: Configure
    trace!("creating display");
    display = Display::new(window_builder, glutin_context_builder, &event_loop)
        .wrap_err("could not initialise display")
        .note(format!("if the error is [NoAvailablePixelFormat] (`{}`), try checking the [glutin::ContextBuilder] settings: vsync, hardware acceleration and srgb may not be a compatible combination on your system", NoAvailablePixelFormat))?;

    trace!("Creating [imgui] context");
    imgui_context = Context::create();
    imgui_context.set_ini_filename(PathBuf::from(log_expr_val!(IMGUI_SETTINGS_FILE_PATH)));
    imgui_context.set_log_filename(PathBuf::from(log_expr_val!(IMGUI_LOG_FILE_PATH)));
    trace!("enabling docking config flag");
    imgui_context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    // trace!("creating font manager");
    // let font_manager = FontManager::new()?;

    //TODO: High DPI setting
    trace!("creating [winit] platform");
    platform = WinitPlatform::init(&mut imgui_context);

    trace!("attaching window");
    platform.attach_window(
        imgui_context.io_mut(),
        display.gl_window().window(),
        HiDpiMode::Default,
    );

    trace!("creating [glium] renderer");
    renderer = Renderer::init(&mut imgui_context, &display).wrap_err("failed to create renderer")?;

    match clipboard_integration::clipboard_init() {
        Ok(clipboard_backend) => {
            trace!("have clipboard support: {clipboard_backend:?}");
            imgui_context.set_clipboard_backend(clipboard_backend);
        }
        Err(error) => {
            warn!("could not initialise clipboard: {error}")
        }
    }

    trace!("done");
    Ok(UiSystem {
        backend: UiBackend {
            event_loop,
            display,
            imgui_context,
            platform,
            renderer,
        },
        managers: UiManagers {
            // font_manager
        },
    })
}