use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError::{Disconnected, Full};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Instant;

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
use tracing::{debug, debug_span, info, instrument, trace, trace_span, warn};

use ProgramThreadMessage::QuitAppNoError;
use QuitAppNoErrorReason::QuitInteractionByUser;

use crate::config::program_config::{APP_TITLE, IMGUI_LOG_FILE_PATH, IMGUI_SETTINGS_FILE_PATH};
use crate::config::ui_config::{
    DEFAULT_WINDOW_SIZE, HARDWARE_ACCELERATION, MULTISAMPLING, START_MAXIMIZED, VSYNC,
};
use crate::helper::logging::event_targets::*;
use crate::helper::logging::log_error;
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::program_messages::{
    unreachable_never_should_be_disconnected, Message, ProgramThreadMessage, QuitAppNoErrorReason,
    UiThreadMessage,
};
use crate::program::ProgramData;
use crate::ui::docking::UiDockingArea;
use crate::ui::font_manager::FontManager;
use crate::ui::ui_system::{UiBackend, UiManagers, UiSystem};
use crate::{log_expr_val, log_variable};

mod clipboard_integration;
mod docking;
mod font_manager;
mod ui_system;

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
    let system = init_ui_system(APP_TITLE).wrap_err("failed while initialising ui system")?;

    // Pulling out the separate variables is the only way I found to avoid getting "already borrowed" errors everywhere
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
    let mut result: eyre::Result<()> = Ok(());
    let result_ref = &mut result;
    let mut last_frame = Instant::now();

    //Enter the imgui_internal span so that any logs will be inside that span by default
    let imgui_internal_span = debug_span!("imgui_internal");
    let _guard_imgui_internal_span = imgui_internal_span.enter();

    debug!("running event loop");
    event_loop.run_return(|event, _window_target, control_flow| {
        macro_rules! event_loop_return {
            ($return_value:expr) => {{
                trace!("expecting event loop to exit");
                *result_ref = $return_value;
                *control_flow = ControlFlow::Exit;
            }};
        }

        let _span = trace_span!(target: UI_PERFRAME_SPAMMY, "process_ui_event", ?event).entered();
        match event {
            //We have new events, but we don't care what they are, just need to update frame timings
            glutin::event::Event::NewEvents(_) => {
                let old_last_frame = last_frame;
                last_frame = Instant::now();
                let delta = last_frame - old_last_frame;
                imgui_context.io_mut().update_delta_time(delta);

                trace!(
                    target: UI_PERFRAME_SPAMMY,
                    "updated deltaT: {}",
                    humantime::format_duration(delta)
                );
            }

            glutin::event::Event::MainEventsCleared => {
                trace!(target: UI_PERFRAME_SPAMMY, "main events cleared");
                trace!(target: UI_PERFRAME_SPAMMY, "requesting redraw");
                let gl_window = display.gl_window();
                let window = gl_window.window();
                window.request_redraw();
                //Pretty sure this makes us render constantly
                trace!(target: UI_PERFRAME_SPAMMY, "preparing frame");
                let result = platform.prepare_frame(imgui_context.io_mut(), window);
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

                let result = outer_render_a_frame(
                    &mut display,
                    &mut imgui_context,
                    &mut platform,
                    &mut renderer,
                    &mut managers,
                );

                if let Err(e) = result {
                    let error = Report::wrap_err(
                        e,
                        "encountered error while drawing: {err}. program should now exit",
                    );
                    warn!("encountered error while drawing");
                    event_loop_return!(Err(error));
                }
            }

            //Handle window events, we just do close events
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                // Here, we don't actually want to close the window, but inform the main thread that we'd like to quit
                // Then, we wait for the main thread to tell us to quit
                info!(target: UI_USER_EVENT, "got CloseRequested window event");

                let message = Program(QuitAppNoError(QuitInteractionByUser));
                debug!("sending message to program thread: {:?}", message);
                match message_sender.try_send(message) {
                    Ok(()) => {
                        // We have signalled the thread, wait till the next loop when the main thread wants us to exit
                        trace!("program thread signalled");
                        trace!("see you on the other side!")
                    }

                    // Neither of these errors should happen ever, but better to be safe
                    Err(Disconnected(_failed_message)) => {
                        let report = Report::msg(
                            "failed to send quit request to program thread: no message receivers",
                        );
                        event_loop_return!(Err(report));
                    }
                    Err(Full(_failed_message)) => {
                        let report = Report::msg(
                            "failed to send quit request to program thread: message buffer full",
                        );
                        event_loop_return!(Err(report));
                    }
                }
            }

            //Catch-all, passes onto the glutin backend
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui_context.io_mut(), gl_window.window(), &event);
            }
        }

        if let Some(ret) = process_messages(&message_sender, &message_receiver) {
            event_loop_return!(ret);
        }

        _span.exit();
    });
    drop(_guard_imgui_internal_span);

    // If we get to here, it's time to exit the thread and shutdown
    info!("ui thread exiting");

    trace!("unsubscribing message receiver");
    message_receiver.unsubscribe();
    trace!("unsubscribing message sender");
    message_sender.unsubscribe();

    Ok(())
}

/// Called every frame, handles everything required to draw an entire frame
///
/// Sets up all the boilerplate, then calls [inner_render] to create (build) the actual ui
fn outer_render_a_frame(
    display: &mut Display,
    imgui_context: &mut Context,
    platform: &mut WinitPlatform,
    renderer: &mut Renderer,
    managers: &mut UiManagers,
) -> color_eyre::Result<()> {
    let _span = trace_span!(
        target: UI_PERFRAME_SPAMMY,
        "outer_render",
        frame = imgui_context.frame_count()
    )
    .entered();

    {
        let mut fonts = imgui_context.fonts();
        match managers.font_manager.rebuild_font_if_needed(&mut fonts) {
            Err(err) => warn!("font manager was not able to rebuild font: {err}"),
            // Font atlas was rebuilt
            Ok(true) => {
                trace!("font was rebuilt, reloading renderer texture");
                let result = renderer.reload_font_texture(imgui_context);
                match result {
                    Ok(()) => trace!("renderer font texture reloaded successfully"),
                    Err(err) => {
                        let report = Report::new(err).wrap_err("could not reload font texture");
                        log_error(&report);
                    }
                }
            }
            Ok(false) => {
                trace!(
                    target: UI_PERFRAME_SPAMMY,
                    "font not rebuilt (probably not dirty)"
                )
            }
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
                    target: UI_PERFRAME_SPAMMY,
                    "font manager failed to return font: {err}"
                );
                None
            }
            Ok(font_id) => {
                trace!(
                    target: UI_PERFRAME_SPAMMY,
                    "got custom font id: {:?}",
                    font_id
                );
                Some(ui.push_font(*font_id))
            }
        };

        let main_window_flags =
            // No borders etc for top-level window
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                // Show menu bar
                | WindowFlags::MENU_BAR
                // Don't raise window on focus (as it'll clobber floating windows)
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS | WindowFlags::NO_NAV_FOCUS
                // Don't want the dock area's parent to be dockable!
                | WindowFlags::NO_DOCKING
            ;

        let _window_padding_token = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
        let main_window_token = ui
            .window("Main Window")
            .flags(main_window_flags)
            .position([0.0, 0.0], Always) // These two make it always fill the whole screen
            .size(ui.io().display_size, Always)
            .begin();
        _window_padding_token.end();
        match main_window_token {
            None => trace!(
                target: UI_PERFRAME_SPAMMY,
                "warning: main window is not visible"
            ),
            Some(token) => {
                let docking_area = UiDockingArea {};
                let _dock_node = docking_area.dockspace("Main Dock Area");

                build_ui(&ui, managers).wrap_err("building ui failed")?;

                token.end();
            }
        }

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
        .wrap_err("ui rendering failed")?;
    target.finish().wrap_err("Failed to swap buffers")?;

    drop(_span);
    return Ok(());
}

fn build_ui(ui: &imgui::Ui, managers: &mut UiManagers) -> eyre::Result<()> {
    let _span = trace_span!(target: UI_PERFRAME_SPAMMY, "build_ui").entered();
    static mut SHOW_DEMO_WINDOW: bool = true;
    static mut SHOW_METRICS_WINDOW: bool = true;
    static mut SHOW_UI_MANAGEMENT_WINDOW: bool = true;
    const NO_SHORTCUT: &str = "N/A";

    trace!(target: UI_PERFRAME_SPAMMY, "processing keybindings");
    macro_rules! key_toggle {
        ($keybinding:ident, $toggle_var:ident) => {
            if ui.is_key_index_pressed_no_repeat($keybinding as i32) {
                $toggle_var ^= true;
                trace!(
                    target: UI_USER_EVENT,
                    "toggle key {} => {}",
                    stringify!($keybinding),
                    $toggle_var
                );
            };
        };
    }
    // key_toggle!(KEY_TOGGLE_METRICS_WINDOW,SHOW_METRICS_WINDOW);
    // key_toggle!(KEY_TOGGLE_DEMO_WINDOW,SHOW_DEMO_WINDOW);
    //
    // trace!(target:UI_PERFRAME_SPAMMY, "menu bar");
    // ui.main_menu_bar(|| {
    //     ui.menu("Tools", ||
    //         {
    //             macro_rules! toggle_menu_item {
    //                     ($item_name:expr, $toggle_var:ident, NO_SHORTCUT) => {
    //                         // Using build_with_ref makes a nice little checkmark appear when the toggle is [true]
    //                          if ui.menu_item_config($item_name).shortcut(NO_SHORTCUT).build_with_ref(&mut $toggle_var){
    //                             trace!(target: UI_USER_EVENT, "toggle menu item {} => {}", stringify!($item_name), $toggle_var);
    //                          }
    //                     };
    //                     ($item_name:expr, $toggle_var:ident, $key:expr) => {
    //                         // Using build_with_ref makes a nice little checkmark appear when the toggle is [true]
    //                          if ui.menu_item_config($item_name).shortcut(format!("{:?}", $key)).build_with_ref(&mut $toggle_var){
    //                             trace!(target: UI_USER_EVENT, "toggle menu item {} => {}", stringify!($item_name), $toggle_var);
    //                          }
    //                     };
    //                 }
    //             toggle_menu_item!("Metrics", SHOW_METRICS_WINDOW, KEY_TOGGLE_METRICS_WINDOW);
    //             toggle_menu_item!("Demo Window", SHOW_DEMO_WINDOW, KEY_TOGGLE_DEMO_WINDOW);
    //             toggle_menu_item!("UI Management", SHOW_UI_MANAGEMENT_WINDOW, NO_SHORTCUT);
    //         });
    // });
    //
    // trace!(target:UI_PERFRAME_SPAMMY, "showing windows");
    // if SHOW_DEMO_WINDOW { ui.show_demo_window(&mut SHOW_DEMO_WINDOW); }
    // if SHOW_METRICS_WINDOW { ui.show_metrics_window(&mut SHOW_METRICS_WINDOW); }
    // managers.render_ui_managers_window(&ui, &mut SHOW_UI_MANAGEMENT_WINDOW);

    ui.show_demo_window(&mut false);

    _span.exit();

    Ok(())
}

/// Function that processes the messages, and returns a value depending on what the UI thread should do
///
/// # Return Value
/// [None] - Do nothing
/// [Some<T>] - UI thread main function should return the value of type T (either [Err()] or [Ok()])
fn process_messages(
    message_sender: &BroadcastSender<Message>,
    message_receiver: &BroadcastReceiver<Message>,
) -> Option<eyre::Result<()>> {
    let _span = trace_span!(
        target: THREAD_MESSAGE_PROCESSING_SPAMMY,
        name_of!(process_messages)
    )
    .entered();
    'loop_messages: loop {
        // Loops until [message_receiver] is empty (tries to 'flush' out all messages)
        let recv = message_receiver.try_recv();
        match recv {
            Err(TryRecvError::Empty) => {
                trace!(
                    target: THREAD_MESSAGE_PROCESSING_SPAMMY,
                    "no messages waiting"
                );
                break 'loop_messages; // Exit the message loop, go into waiting
            }
            Err(TryRecvError::Disconnected) => {
                unreachable_never_should_be_disconnected();
            }
            Ok(message) => {
                trace!(
                    target: THREAD_MESSAGE_PROCESSING_SPAMMY,
                    "got message: {:?}",
                    &message
                );
                match message {
                    Program(_program_message) => {
                        trace!(
                            target: THREAD_MESSAGE_PROCESSING_SPAMMY,
                            "[ui] message for program thread, ignoring"
                        );
                        continue 'loop_messages;
                    }
                    Engine(_engine_message) => {
                        trace!(
                            target: THREAD_MESSAGE_PROCESSING_SPAMMY,
                            "[ui] message for engine thread, ignoring"
                        );
                        continue 'loop_messages;
                    }
                    Ui(ui_message) => {
                        return match ui_message {
                            UiThreadMessage::ExitUiThread => {
                                trace!("got exit message for Ui thread");
                                Some(Ok(())) //Ui thread should return with Ok
                            }
                        };
                    }
                }
            }
        }
    }

    //UI thread should keep running
    None
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
    event_loop = EventLoopBuilder::with_any_thread(&mut EventLoopBuilder::new(), true).build();

    trace!("creating [glutin] context builder");
    let glutin_context_builder = glutin::ContextBuilder::new() //TODO: Configure
        .with_vsync(VSYNC)
        .with_hardware_acceleration(HARDWARE_ACCELERATION)
        .with_srgb(true)
        .with_multisampling(MULTISAMPLING);
    log_variable!(glutin_context_builder:?);

    trace!("creating [winit] window builder");
    let window_builder = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(DEFAULT_WINDOW_SIZE)
        .with_maximized(START_MAXIMIZED);
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

    trace!("creating font manager");
    let font_manager = FontManager::new().wrap_err("failed to create font manager")?;

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
    renderer =
        Renderer::init(&mut imgui_context, &display).wrap_err("failed to create renderer")?;

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
        managers: UiManagers { font_manager },
    })
}
