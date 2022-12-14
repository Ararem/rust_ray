use std::sync::mpsc::TrySendError::*;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use color_eyre::eyre::WrapErr;
use color_eyre::{eyre, Help, Report, SectionExt};
use indoc::formatdoc;
use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug, debug_span, error, info, info_span, trace, trace_span};

use program_data::ProgramData;
use ProgramThreadMessage::{QuitAppError, QuitAppNoError};
use QuitAppNoErrorReason::QuitInteractionByUser;

use crate::engine::*;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::{dyn_panic_to_report, format_report_display, format_report_string};
use crate::program::thread_messages::ThreadMessage::*;
use crate::program::thread_messages::*;
use crate::ui::ui_data::UiData;
use crate::ui::*;
use crate::FallibleFn;

#[macro_use]
pub(crate) mod thread_messages;
pub mod program_data;

pub type ThreadReturn = FallibleFn;
pub type ThreadHandle = JoinHandle<ThreadReturn>;

pub fn run() -> ThreadReturn {
    let span_run = info_span!(target: PROGRAM_INFO_LIFECYCLE, name_of!(run)).entered();

    let span_init = debug_span!(target: PROGRAM_DEBUG_GENERAL, "program_init").entered();
    // Create new program 'instance'
    debug!(target: PROGRAM_DEBUG_GENERAL, "creating ProgramData");
    let program_data = ProgramData {
        ui_data: UiData::default(),
        engine_data: EngineData {},
    };
    debug!(target: PROGRAM_DEBUG_GENERAL, ?program_data);

    // Wrap the program data inside an Arc(Mutex(T))
    // This allows us to:
    // (Arc): Share a reference of the Mutex(ProgramData) across the threads safely
    // (Mutex): Use that reference to give a single thread access to the ProgramData at one time
    debug!(target: PROGRAM_DEBUG_GENERAL, "wrapping program data for thread-safety");
    let program_data_wrapped = Arc::new(Mutex::new(program_data));
    debug!(target: PROGRAM_DEBUG_GENERAL, ?program_data_wrapped);

    // The engine/ui threads use the command_sender to send messages back to the main thread, in order to do stuff (like quit the app)
    debug!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "creating MPMC channel for thread communication");
    let (msg_sender, msg_receiver) = broadcast_queue::<ThreadMessage>(100);
    debug!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "created MPMC channel");

    // This barrier blocks our UI and engine thread from starting until the program is ready for them
    debug!(target: THREAD_DEBUG_GENERAL, "creating thread start barrier for threads");
    let thread_start_barrier = Arc::new(Barrier::new(3));
    // 3 = 1 (engine) + 1 (ui) + 1 (main thread)
    debug!(target: THREAD_DEBUG_GENERAL, "created thread start barrier");

    span_init.exit();

    let mut threads: Threads = debug_span!(target: THREAD_DEBUG_GENERAL, "create_threads").in_scope(|| -> eyre::Result<Threads> {
        debug!(target: THREAD_DEBUG_GENERAL, "creating engine thread");
        let engine_thread_handle: ThreadHandle = {
            let data = Arc::clone(&program_data_wrapped);
            let sender = msg_sender.clone();
            let receiver = msg_receiver.add_stream();
            let barrier = Arc::clone(&thread_start_barrier);
            thread::Builder::new()
                .name("engine_thread".to_string())
                .spawn(move || engine_thread(barrier, data, sender, receiver))
                .wrap_err("failed to create engine thread")
                .note("this error was most likely due to a failure at the OS level")?
        };
        debug!(target: THREAD_DEBUG_GENERAL, ?engine_thread_handle, "created engine thread");

        debug!(target: THREAD_DEBUG_GENERAL, "creating ui thread");
        let ui_thread_handle: ThreadHandle = {
            let data = Arc::clone(&program_data_wrapped);
            let sender = msg_sender.clone();
            let receiver = msg_receiver.add_stream();
            let barrier = Arc::clone(&thread_start_barrier);
            thread::Builder::new()
                .name("ui_thread".to_string())
                .spawn(|| ui_thread(barrier, data, sender, receiver))
                .wrap_err("failed to create ui thread")
                .note("this error was most likely due to a failure at the OS level")?
        };
        debug!(target: THREAD_DEBUG_GENERAL, ?ui_thread_handle, "created ui thread");

        debug!(target: THREAD_DEBUG_GENERAL, "waiting on barrier to enable it");
        thread_start_barrier.wait();
        debug!(target: THREAD_DEBUG_GENERAL, "threads should now be awake");
        Ok(Threads {
            engine: engine_thread_handle,
            ui: ui_thread_handle,
        })
    })?;

    let poll_interval = Duration::from_millis(1000);
    // Should loop until program exits
    debug!(target: PROGRAM_DEBUG_GENERAL, ?poll_interval, "entering 'global loop");

    let span_global_loop = debug_span!(target: PROGRAM_DEBUG_GENERAL, "'global").entered();
    'global: for global_iter in 0usize.. {
        let span_global_loop_inner = trace_span!(target: PROGRAM_TRACE_GLOBAL_LOOP, "inner", %global_iter).entered();

        // Process any messages we might have from the other threads
        let span_process_messages = trace_span!(target: THREAD_TRACE_MESSAGE_LOOP, "process_messages").entered();
        'process_messages: loop {
            if let Some(message) = receive_message(&msg_receiver)? {
                match message {
                    Ui(_) | Engine(_) => {
                        message.ignore();
                        continue 'process_messages;
                    }
                    Program(program_message) => {
                        debug!(target: THREAD_DEBUG_MESSAGE_RECEIVED, ?program_message, "got program message");
                        match program_message {
                            QuitAppNoError(QuitInteractionByUser) => {
                                handle_user_quit(msg_sender, msg_receiver, threads)?;
                                break 'global;
                            }
                            QuitAppError(wrapped_error_report) => return Err(handle_error_quit(wrapped_error_report)),
                        }
                    }
                }
            }
            // No messages waiting
            else {
                break 'process_messages;
            }
        } //end 'loop_messages
        span_process_messages.exit();

        /*
        Ensure the threads are OK (still running)
        They should only ever safely exit while inside the 'process_messages loop (since that's where they're told to quit)
        So if they have finished here, that's BAAADDDD
        */
        threads = check_threads_are_running(threads).wrap_err("failed thread status check")?;

        trace!(target: PROGRAM_TRACE_GLOBAL_LOOP, ?poll_interval, "sleeping");
        thread::sleep(poll_interval);
        span_global_loop_inner.exit();
    } //end 'global
    span_global_loop.exit();

    debug!(target: PROGRAM_DEBUG_GENERAL, "program 'global loop finished");

    span_run.exit();
    Ok(())
}

struct Threads {
    engine: ThreadHandle,
    ui: ThreadHandle,
}

fn check_threads_are_running(threads: Threads) -> eyre::Result<Threads> {
    let span_check_threads = trace_span!(target: PROGRAM_TRACE_THREAD_STATUS_POLL, "check_threads").entered();
    trace!(target: PROGRAM_TRACE_THREAD_STATUS_POLL, "checking ui thread status");
    if threads.ui.is_finished() {
        error!(target: THREAD_DEBUG_GENERAL, "ui thread finished early when it shouldn't have, joining to get return value");
        // Thread finished so .join() should be wait-free
        return match threads.ui.join() {
            Ok(thread_return) => {
                let formatted_thread_return = match thread_return {
                    Ok(()) => "Ok(())".to_string(),
                    Err(report) => format_report_string(&report).replace("\n", "\n||\t\t"),
                };
                let error = Report::msg("ui thread finished early")
                    .section(format!("Return Value:\n{formatted_thread_return}"));
                Err(error)
            }
            Err(boxed_panic) => {
                let error = dyn_panic_to_report(&boxed_panic).wrap_err("ui thread panicked while running");
                Err(error)
            }
        };
    } else {
        trace!(target: PROGRAM_TRACE_THREAD_STATUS_POLL, "ui thread still running");
    }

    trace!(target: PROGRAM_TRACE_THREAD_STATUS_POLL, "checking engine thread status");
    if threads.engine.is_finished() {
        error!(target: THREAD_DEBUG_GENERAL, "engine thread finished early when it shouldn't have, joining to get return value");
        // Thread finished so .join() should be wait-free
        return match threads.engine.join() {
            Ok(thread_return) => {
                let formatted_thread_return = match thread_return {
                    Ok(()) => "Ok(())".to_string(),
                    Err(report) => format_report_string(&report).replace("\n", "\n||\t\t"),
                };
                let error = Report::msg("engine thread finished early")
                    .section(format!("Return Value:\n{formatted_thread_return}"));
                Err(error)
            }
            Err(boxed_panic) => {
                let error = dyn_panic_to_report(&boxed_panic).wrap_err("engine thread panicked while running");
                debug!(target: THREAD_DEBUG_GENERAL, report=%format_report_display(&error));
                Err(error)
            }
        };
    } else {
        trace!(target: PROGRAM_TRACE_THREAD_STATUS_POLL, "engine thread still running");
    }

    span_check_threads.exit();
    Ok(threads)
}

fn handle_error_quit(wrapped_error_report: Arc<Report>) -> Report {
    info!(target: PROGRAM_INFO_LIFECYCLE, "quitting app due to error");
    // try and return the error directly, but just in case we can't (which should never happen), print the error and return a generic one
    match Arc::try_unwrap(wrapped_error_report) {
        //Only one strong reference to the arc ([wrapped_error_report]), so we got ownership
        // Should always happen
        Ok(owned_error) => owned_error.wrap_err("quitting app due to internal error"),
        Err(_arc) => {
            let warn = formatdoc! {"
                                        was unable to unwrap error report for quitting app - there are {} strong references (should be only 1).
                                        this should not happen, there is a bug in the error creation code.
                                        ", Arc::strong_count(&_arc)
            };
            error!(target: REALLY_FUCKING_BAD_UNREACHABLE, "{}", warn);
            Report::msg("quitting app due to an internal error")
                .with_section(move || format_report_display(&_arc).header("Error:"))
                .note("the displayed error may not be correct and/or complete")
                .warning(warn)
        }
    }
}

fn handle_user_quit(message_sender: BroadcastSender<ThreadMessage>, message_receiver: BroadcastReceiver<ThreadMessage>, threads: Threads) -> FallibleFn {
    info!(target: PROGRAM_INFO_LIFECYCLE, "user wants to quit");

    // We have to unsubscribe from out receiver or it blocks the other threads because we haven't received the [ExitXXXThread] messages
    trace!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "unsubbing (program) message receiver to release stream");
    message_receiver.unsubscribe();
    trace!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "unsubscribed (program) message receiver");
    debug_span!(target: THREAD_DEBUG_GENERAL, "join_threads_and_quit").in_scope(|| {
        debug_span!(target: THREAD_DEBUG_GENERAL, "stop_ui").in_scope(|| {
            let message = Ui(UiThreadMessage::ExitUiThread);
            debug!(target: THREAD_DEBUG_MESSAGE_SEND, ?message);
            match message_sender.try_send(message) {
                Ok(()) => {
                    debug!(target: THREAD_DEBUG_GENERAL, "ui thread signalled, joining threads");
                    let join_result = threads.ui.join();
                    debug!(target: THREAD_DEBUG_GENERAL, ?join_result, "ui thread joined");
                    match join_result {
                        // Thread joined normally, [thread_return_value] is what the thread returned
                        Ok(thread_return_value) => {
                            match thread_return_value {
                                Ok(return_value) => {
                                    debug!(target: THREAD_DEBUG_GENERAL, ?return_value, "ui thread completed successfully");
                                }
                                Err(error) => {
                                    // The ui thread failed while shutting down here
                                    // If it failed normally then it would have been caught outside the 'process_messages loop
                                    let error = error
                                        .wrap_err("ui thread failed while shutting down")
                                        .note("it is unlikely that the thread failed during normal execution, as that should have been caught earlier");
                                    debug!(target: THREAD_DEBUG_GENERAL, ?error);
                                    //TODO: Test how this error code works and is logged
                                    return Err(error);
                                }
                            }
                        }
                        // Thread panicked while quitting
                        Err(boxed_panic) => {
                            // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                            // So we have to format it as a string
                            let report = dyn_panic_to_report(&boxed_panic)
                                .wrap_err("ui thread panicked while shutting down")
                                .note("it is unlikely that the thread failed during normal execution, as that should have been caught earlier");
                            debug!(target: THREAD_DEBUG_GENERAL, ?boxed_panic, ?report);
                            return Err(report);
                        }
                    }
                }

                // Neither of these errors should happen ever, but better to be safe
                Err(Disconnected(_failed_message)) => {
                    return Err(error_recv_never_should_be_disconnected().note(format!("attempted to send quit signal to ui thread: {_failed_message:?}")));
                }
                Err(Full(_failed_message)) => {
                    return Err(error_recv_never_should_be_disconnected().note(format!("attempted to send quit signal to ui thread: {_failed_message:?}")));
                }
            }

            Ok(())
        })?; //end stop_ui

        debug_span!(target: THREAD_DEBUG_GENERAL, "stop_engine").in_scope(|| {
            match message_sender.try_send(Engine(EngineThreadMessage::ExitEngineThread)) {
                Ok(()) => {
                    debug!(target: THREAD_DEBUG_GENERAL, "engine thread signalled, joining threads");
                    let join_result = threads.engine.join();
                    debug!(target: THREAD_DEBUG_GENERAL, ?join_result, "engine thread joined");
                    match join_result {
                        // Thread joined normally, [thread_return_value] is what the thread returned
                        Ok(thread_return_value) => {
                            match thread_return_value {
                                Ok(return_value) => {
                                    debug!(target: THREAD_DEBUG_GENERAL, ?return_value, "engine thread completed successfully");
                                }
                                Err(error) => {
                                    // The engine thread failed while shutting down here
                                    // If it failed normally then it would have been caught outside the 'process_messages loop
                                    let error = error
                                        .wrap_err("engine thread failed while shutting down")
                                        .note("it is unlikely that the thread failed during normal execution, as that should have been caught earlier");
                                    debug!(target: THREAD_DEBUG_GENERAL, ?error);
                                    //TODO: Test how this error code works and is logged
                                    return Err(error);
                                }
                            }
                        }
                        // Thread panicked while quitting
                        Err(boxed_panic) => {
                            // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                            // So we have to format it as a string
                            let report = dyn_panic_to_report(&boxed_panic).wrap_err("engine thread panicked while shutting down");
                            debug!(target: THREAD_DEBUG_GENERAL, ?boxed_panic, ?report);
                            return Err(report);
                        }
                    }
                }

                // Neither of these errors should happen ever, but better to be safe
                Err(Disconnected(_failed_message)) => {
                    return Err(error_recv_never_should_be_disconnected().note(format!("attempted to send quit signal to engine thread: {_failed_message:?}")));
                }
                Err(Full(_failed_message)) => {
                    return Err(error_recv_never_should_be_disconnected().note(format!("attempted to send quit signal to engine thread: {_failed_message:?}")));
                }
            }

            Ok(())
        })?; //end stop_engine

        // We know all is well if we get here, since we return immediately on any error when joining
        debug!(target: THREAD_DEBUG_GENERAL, "engine and ui threads joined successfully");

        Result::<(), Report>::Ok(())
    })?;

    Ok(())
}
