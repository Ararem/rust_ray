use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use flume::{Receiver, Sender};
use nameof::name_of;
use tracing::{info, instrument, trace};

use crate::helper::logging::event_targets::*;
use crate::program::program_messages::{EngineThreadMessage, Message};
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::ProgramData;

#[derive(Copy, Clone, Debug)]
pub struct EngineData {}

#[instrument(ret, skip_all)]
pub(crate) fn engine_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: Sender<Message>,
    message_receiver: Receiver<Message>,
) {
    //Create a NoPanicPill to make sure we DON'T PANIC
    let _no_panic_pill = crate::helper::no_panic_pill::NoPanicPill {};

    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running engine thread");

    loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        'loop_messages: loop {
            // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
            let recv = message_receiver.try_recv();
            match recv {
                Err(flume::TryRecvError::Empty) => {
                    trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "no messages waiting");
                    break 'loop_messages; // Exit the message loop, go into waiting
                }
                Err(flume::TryRecvError::Disconnected) => {
                    // Should (only) get here once the UI and engine threads have exited, and therefore their closures have dropped the sender variables
                    trace!("all senders have disconnected from program message channel");
                    todo!("ALL SENDERS DISCONNECTED HANDLING");
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
                                "[engin] message for ui thread, ignoring"
                            );
                            continue 'loop_messages;
                        }
                        Program(_program_message) => {
                            trace!(
                                target: PROGRAM_MESSAGE_POLL_SPAMMY,
                                "[engine] message for program thread, ignoring"
                            );
                            continue 'loop_messages;
                        }
                        Engine(engine_message) => match engine_message {
                            EngineThreadMessage::ExitEngineThread => {
                                info!("got exit message for Ui thread");
                                return;
                            }
                        },
                    }
                }
            }
        }
    }

    drop(_no_panic_pill);
}
