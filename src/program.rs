use std::ops::Deref;
use std::sync::mpsc::{TryRecvError, TrySendError::*};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use color_eyre::{eyre, Help, Report};
use indoc::formatdoc;
use multiqueue2::broadcast_queue;
use nameof::name_of_type;
use tracing::{debug, debug_span, error, info, instrument, trace, warn};

use crate::engine::*;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::log_error;
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::program_messages::*;
use crate::ui::*;

#[macro_use]
pub(crate) mod program_messages;

/// Main data structure used
#[derive(Debug)]
pub struct ProgramData {
    pub ui_data: UiData,
    pub engine_data: EngineData,
}

#[instrument]
pub fn run() -> eyre::Result<()> {
    // Create new program instance
    trace!("creating {} struct", name_of_type!(ProgramData));
    let program_data = ProgramData {
        ui_data: UiData {},
        engine_data: EngineData {},
    };

    // Wrap the program data inside an Arc(Mutex(T))
    // This allows us to:
    // (Arc): Share a reference of the Mutex(ProgramData) across the threads safely
    // (Mutex): Use that reference to give a single thread access to the ProgramData at one time
    trace!("wrapping program data for thread-safety");
    let program_data_wrapped = Arc::new(Mutex::new(program_data));

    // The engine/ui threads use the command_sender to send messages back to the main thread, in order to do stuff (like quit the app)
    trace!("creating MPMC channel for thread communication");
    let (message_sender, message_receiver) = broadcast_queue::<Message>(0);

    // This barrier blocks our UI and engine thread from starting until the program is ready for them
    trace!("creating start thread barrier for threads");
    let thread_start_barrier = Arc::new(Barrier::new(3)); // 3 = 1 (engine) + 1 (ui) + 1 (main thread)

    debug!("creating engine thread");
    let engine_thread: JoinHandle<()> = {
        let data = Arc::clone(&program_data_wrapped);
        let sender = message_sender.clone();
        let receiver = message_receiver.add_stream();
        let barrier = Arc::clone(&thread_start_barrier);
        match thread::Builder::new()
            .name("engine_thread".to_string())
            .spawn(move || engine_thread(barrier, data, sender, receiver))
        {
            Ok(thread) => thread,
            Err(error) => {
                let report = Report::new(error)
                    .wrap_err("failed to create engine thread")
                    .note("this error was most likely due to a failure at the OS level");

                return Err(report);
            }
        }
    };
    trace!("created engine thread");

    debug!("creating ui thread");
    let ui_thread: JoinHandle<eyre::Result<()>> = {
        let data = Arc::clone(&program_data_wrapped);
        let sender = message_sender.clone();
        let receiver = message_receiver.add_stream();
        let barrier = Arc::clone(&thread_start_barrier);
        match thread::Builder::new()
            .name("ui_thread".to_string())
            .spawn(move || ui_thread(barrier, data, sender, receiver))
        {
            Ok(thread) => thread,
            Err(error) => {
                let report = Report::new(error)
                    .wrap_err("failed to create ui thread")
                    .note("this error was most likely due to a failure at the OS level");

                return Err(report);
            }
        }
    };
    trace!("created ui thread");

    trace!("waiting on barrier to enable it");
    thread_start_barrier.wait();
    trace!("threads should now be awake");

    let mut return_value;
    let poll_interval = Duration::from_millis(1000);
    // Should loop until program exits
    'global: loop {
        // Process any messages we might have from the other threads
        let process_messages_span = debug_span!("process_messages").entered();
        'loop_messages: loop {
            // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
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
                        Ui(_ui_message) => {
                            trace!(
                                target: PROGRAM_MESSAGE_POLL_SPAMMY,
                                "[program] message for ui thread, ignoring"
                            );
                            continue 'loop_messages;
                        }
                        Engine(_engine_message) => {
                            trace!(
                                target: PROGRAM_MESSAGE_POLL_SPAMMY,
                                "[program] message for engine thread, ignoring"
                            );
                            continue 'loop_messages;
                        }
                        Program(program_message) => match program_message {
                            ProgramThreadMessage::QuitAppNoError(
                                QuitAppNoErrorReason::QuitInteractionByUser,
                            ) => {
                                info!("user wants to quit");

                                // We have to unsubscribe from out receiver or it blocks the other threads because we haven't received the [ExitXXXThread] messages
                                trace!("unsubbing message receiver to release stream");
                                message_receiver.unsubscribe();
                                let _join_threads_span =
                                    debug_span!("join_threads_and_quit").entered();

                                debug!("signalling ui thread to quit");
                                match message_sender.try_send(Ui(UiThreadMessage::ExitUiThread)) {
                                    Ok(()) => {
                                        trace!("ui thread signalled, joining threads");
                                        let join_result = ui_thread.join();
                                        match join_result {
                                            Ok(return_value) => {
                                                trace!("ui thread joined");
                                                match return_value {
                                                    Ok(()) => {
                                                        debug!("ui thread return was Ok");
                                                    }
                                                    Err(error) => {
                                                        log_error(&error.wrap_err(
                                                            "ui thread exited with error",
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(boxed_error) => {
                                                // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                                                // So we have to format it as a string
                                                let report = Report::msg(format!(
                                                    "ui thread panicked:\n{:#?}",
                                                    boxed_error
                                                ));
                                                return Err(report);
                                            }
                                        }
                                    }

                                    // Neither of these errors should happen ever, but better to be safe
                                    Err(Disconnected(_failed_message)) => {
                                        let report = Report::msg("failed to send quit signal to ui thread: no message receivers");
                                        return Err(report);
                                    }
                                    Err(Full(_failed_message)) => {
                                        let report = Report::msg("failed to send quit signal to ui thread: message buffer full");
                                        return Err(report);
                                    }
                                }

                                debug!("signalling engine thread to quit");
                                match message_sender
                                    .try_send(Engine(EngineThreadMessage::ExitEngineThread))
                                {
                                    Ok(()) => {
                                        trace!("engine thread signalled, joining threads");
                                        let join_result = engine_thread.join();
                                        match join_result {
                                            Ok(return_value) => {
                                                trace!("engine thread joined");
                                                warn!("TODO: engine thread return value");
                                                // match return_value {
                                                // Ok(()) => {
                                                //     debug!("ui thread return was Ok");
                                                // }
                                                // Err(error) => {
                                                //     log_error(&error.wrap_err(
                                                //         "engine thread exited with error",
                                                //     ));
                                                // }
                                                // }
                                            }
                                            Err(boxed_error) => {
                                                // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                                                // So we have to format it as a string
                                                let report = Report::msg(format!(
                                                    "engine thread panicked:\n{:#?}",
                                                    boxed_error
                                                ));
                                                return Err(report);
                                            }
                                        }
                                    }

                                    // Neither of these errors should happen ever, but better to be safe
                                    Err(Disconnected(..)) => {
                                        let report = Report::msg("failed to send quit signal to engine thread: no message receivers");
                                        return Err(report);
                                    }
                                    Err(Full(..)) => {
                                        let report = Report::msg("failed to send quit signal to engine thread: message buffer full");
                                        return Err(report);
                                    }
                                }

                                // We know all is well if we get here, since we return immediately on any error when joining
                                trace!("engine and ui threads joined successfully");
                                drop(_join_threads_span);

                                return_value = Ok(());
                                break 'global;
                            }
                            ProgramThreadMessage::QuitAppError(wrapped_error_report) => {
                                info!("quitting app due to error");
                                // try and return the error directly, but just in case we can't (which should never happen), print the error and return a generic one
                                return match Arc::try_unwrap(wrapped_error_report) {
                                    //Only one strong reference to the arc ([wrapped_error_report]), so we got ownership
                                    // Should always happen
                                    Ok(owned_error) => {
                                        Err(owned_error
                                            .wrap_err("quitting app due to internal error"))
                                    }
                                    Err(_arc) => {
                                        let warn = formatdoc! {"
                                        was unable to unwrap error report for quitting app - there are {} strong references (should be only 1).
                                        this should not happen, there is a bug in the error creation code.
                                        ", Arc::strong_count(&_arc)
                                        };
                                        warn!("{}", warn);
                                        log_error(&_arc);
                                        Err(Report::msg("quitting app due to an internal error (cannot display due to rust ownership)")
                                            .warning(warn))
                                    }
                                };
                            }
                        },
                    }
                }
            }
        } //end 'loop_messages
        drop(process_messages_span);

        /*
        Ensure the threads are OK (still running)
        They should only ever safely exit while inside the 'process_messages loop (since that's where they're told to quit)
        So if they have finished here, that's BAAADDDD
        */
        trace!(target: PROGRAM_RUN_LOOP_SPAMMY, "checking ui thread status");
        if ui_thread.is_finished() {
            error!("ui thread finished early when it shouldn't have, joining to get return value");
            let join_result = ui_thread.join();
            return Err(match join_result {
                Ok(ret) => {
                    let report = Report::msg(format!(
                        "ui thread finished early with return value `{:#?}`",
                        ret
                    ));
                    report
                }
                Err(boxed_error) => {
                    let report = Report::msg(format!("ui thread panicked:\n{:#?}", boxed_error));
                    report
                }
            });
        } else {
            trace!(target: PROGRAM_RUN_LOOP_SPAMMY, "ui thread still running");
        }
        trace!(
            target: PROGRAM_RUN_LOOP_SPAMMY,
            "checking engine thread status"
        );
        if engine_thread.is_finished() {
            error!("engine finished early when it shouldn't have, joining to get return value");
            let join_result = engine_thread.join();
            return Err(match join_result {
                Ok(ret) => {
                    let report = Report::msg(format!(
                        "engine thread finished early with return value `{:#?}`",
                        ret
                    ));
                    report
                }
                Err(boxed_error) => {
                    let report =
                        Report::msg(format!("engine thread panicked:\n{:#?}", boxed_error));
                    report
                }
            });
        } else {
            trace!(
                target: PROGRAM_RUN_LOOP_SPAMMY,
                "engine thread still running"
            );
        }
        thread::sleep(poll_interval);
    } //end 'global

    debug!("program 'global loop finished");

    return return_value;
}
