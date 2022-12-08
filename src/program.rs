use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::{TryRecvError, TrySendError::*};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use color_eyre::{eyre, Help, Report};
use multiqueue2::broadcast_queue;
use nameof::name_of_type;
use tracing::{debug, debug_span, error, info, instrument, trace};

use crate::engine::*;
use crate::helper::logging::event_targets::*;
use crate::program::program_messages::*;
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::ui::*;

#[macro_use]
pub(crate) mod program_messages;

/// Main data structure used
#[derive(Debug)]
pub struct ProgramData {
    pub ui_data: UiData,
    pub engine_data: EngineData,
}

#[instrument(ret)]
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
    let ui_thread: JoinHandle<()> = {
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

    // Process messages in a loop
    let poll_interval = Duration::from_millis(1000);
    // Should loop until program exits
    'global: loop {
        // Process any messages we might have from the other threads
        let _span = debug_span!("process_messages").entered();
        'loop_messages: loop {
            // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
            let recv = message_receiver.try_recv();
            match recv {
                Err(TryRecvError::Empty) => {
                    trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "no messages waiting");
                    break 'loop_messages; // Exit the message loop, go into waiting
                }
                Err(TryRecvError::Disconnected) => {
                    program_thread_messaging__unreachable_never_should_be_disconnected!();
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
                                let _join_threads_span = debug_span!("join_threads_and_quit").entered();

                                trace!("signalling ui thread to quit");
                                match message_sender.try_send(Ui(UiThreadMessage::ExitUiThread)) {
                                    Ok(()) => {
                                        trace!("ui thread signalled, joining threads");
                                        let join_result = ui_thread.join();
                                        match join_result {
                                            Ok(()) => {
                                                trace!("ui thread joined successfully");
                                            },
                                            Err(boxed_error) => {
                                                // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                                                // So log the error, and pass up a generic one to the main function
                                                error!("ui thread joined with error: {boxed_error:#?}");
                                                let report = Report::msg("ui thread panicked (cannot display error due to [Box<dyn...>])");
                                                return Err(report);
                                            }
                                        }
                                    },

                                    // Neither of these errors should happen ever, but better to be safe
                                    Err(Disconnected(..)) => {
                                        let report = Report::msg("failed to send quit signal to ui thread: no message receivers");
                                        return Err(report);
                                    },
                                    Err(Full(..)) => {
                                        let report = Report::msg("failed to send quit signal to ui thread: message buffer full");
                                        return Err(report);
                                    }
                                }

                                trace!("signalling engine thread to quit");
                                match message_sender.try_send(Engine(EngineThreadMessage::ExitEngineThread)) {
                                    Ok(()) => {
                                        trace!("engine thread signalled, joining threads");
                                        let join_result = engine_thread.join();
                                        match join_result {
                                            Ok(()) => {
                                                trace!("engine thread joined successfully");
                                            },
                                            Err(boxed_error) => {
                                                // Unfortunately we can't use the error for a report since it doesn't implement Sync, and it's dyn
                                                // So log the error, and pass up a generic one to the main function
                                                error!("engine thread joined with error: {boxed_error:#?}");
                                                let report = Report::msg("engine thread panicked (cannot display error due to [Box<dyn...>])");
                                                return Err(report);
                                            }
                                        }
                                    },

                                    // Neither of these errors should happen ever, but better to be safe
                                    Err(Disconnected(..)) => {
                                        let report = Report::msg("failed to send quit signal to engine thread: no message receivers");
                                        return Err(report);
                                    },
                                    Err(Full(..)) => {
                                        let report = Report::msg("failed to send quit signal to engine thread: message buffer full");
                                        return Err(report);
                                    }
                                }

                                // We know all is well if we get here, since we return immediately on any error when joining
                                trace!("engine and ui threads joined successfully");
                                drop(_join_threads_span);

                                break 'global;
                            }
                            ProgramThreadMessage::QuitAppError(error_report) => {
                                info!("quitting app due to error");
                                error!("{}", error_report);
                                break 'global;
                            }
                        },
                    }
                }
            }
        }
        drop(_span);

        // Ensure the threads are OK
        // ui_thread.thread().
        thread::sleep(poll_interval);
    }

    trace!("hello?!");
    thread::sleep(Duration::from_secs(1));

    return Ok(());
}
