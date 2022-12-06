use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use color_eyre::{eyre, Help, Report};
use nameof::{name_of, name_of_type};
use tracing::{debug, error, info, instrument, trace};

use flume::*;

use crate::engine::*;
use crate::helper::logging::event_targets::*;
use crate::program::program_messages::*;
use crate::ui::*;

pub(crate) mod program_messages;

/// Main data structure used
#[derive(Debug)]
pub struct ProgramData {
    pub ui_data: UiData,
    pub engine_data: EngineData
}

#[instrument(ret)]
pub fn run() -> eyre::Result<()> {
    // Create new program instance
    trace!("creating {} struct", name_of_type!(ProgramData));
    let program_data = ProgramData {
        ui_data: UiData {},
        engine_data: EngineData {}
    };

    // Wrap the program data inside an Arc(Mutex(T))
    // This allows us to:
    // (Arc): Share a reference of the Mutex(ProgramData) across the threads safely
    // (Mutex): Use that reference to give a single thread access to the ProgramData at one time
    trace!("wrapping program data for thread-safety");
    let program_data_wrapped = Arc::new(Mutex::new(program_data));

    // The engine/ui threads use the command_sender to send messages back to the main thread, in order to do stuff (like quit the app)
    trace!("creating mpsc channel for thread communication");
    let (command_sender, command_receiver) = flume::unbounded::<ProgramMessage>();

    // This barrier blocks our UI and engine thread from starting until the program is ready for them
    trace!("creating start thread barrier for threads");
    let thread_start_barrier = Arc::new(Barrier::new(3)); // 3 = 1 (engine) + 1 (ui) + 1 (main thread)

    debug!("creating engine thread");
    let engine_thread =
        {
            let data = Arc::clone(&program_data_wrapped);
            let sender = Sender::clone(&command_sender);
            let barrier = Arc::clone(&thread_start_barrier);
            match thread::Builder::new()
                .name("engine_thread".to_string())
                .spawn(move || engine_thread(barrier, data, sender)) {
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
    let ui_thread = {
        let data = Arc::clone(&program_data_wrapped);
        let sender = flume::Sender::clone(&command_sender);
        let barrier = Arc::clone(&thread_start_barrier);
        match thread::Builder::new()
            .name("ui_thread".to_string())
            .spawn(move || ui_thread(barrier, data, sender)) {
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

    // Drop command sender now that we've sent it to the threads
    drop(command_sender);

    trace!("waiting on barrier to enable it");
    thread_start_barrier.wait();
    trace!("threads should now be awake");

    // Process messages in a loop
    let poll_interval = Duration::from_millis(1000);
    // Should loop until program exits
    'global: loop {
        trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "checking {} for messages", name_of!(command_receiver));
        'loop_messages: loop { // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
            let recv = command_receiver.try_recv();
            match recv {
                Err(Empty) => {
                    trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "no messages waiting");
                    break 'loop_messages; // Exit the message loop, go into waiting
                },
                Err(Disconnected) => {
                    // Should (only) get here once the UI and engine threads have exited, and therefore their closures have dropped the sender variables
                    trace!("all senders have disconnected from program message channel");
                    todo!("ALL SENDERS DISCONNECTED HANDLING");
                },
                Ok(message) => {
                    trace!(target: PROGRAM_MESSAGE_POLL, "got {}: {:?}", name_of_type!(ProgramMessage), message);
                    match message {
                        ProgramMessage::QuitAppNoError(QuitAppNoErrorReason::QuitInteractionByUser) => {
                            info!("user wants to quit");
                            todo!("Quit handling");
                        },
                        ProgramMessage::QuitAppError(QuitAppErrorReason::Error(error_report)) => {
                            info!("quitting app due to error");
                            error!("{}", error_report);
                            break 'global;
                        }
                    }
                }
            }
        }

        thread::sleep(poll_interval);
    }


    trace!("hello?!");
    thread::sleep(Duration::from_secs(1));

    return Ok(());
}
