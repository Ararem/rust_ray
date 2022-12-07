use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug_span, info, instrument, trace};

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
    message_sender: BroadcastSender<Message>,
    message_receiver: BroadcastReceiver<Message>,
) {
    //Create a NoPanicPill to make sure we DON'T PANIC
    let _no_panic_pill = crate::helper::panic_pill::PanicPill {};

    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running engine thread");

    'global: loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

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
                    // Should (only) get here once the program and engine threads have exited, and therefore they have dropped their sender variables
                    // This is not supposed to happen, since the program thread should always be the last to exit
                    unreachable!(r"engine thread {} returned [Disconnected], which shouldn't be possible (the program thread should always be alive while the engine thread is alive)", name_of!(message_receiver));
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
                                "[engine] message for ui thread, ignoring"
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
                                info!("got exit message for engine thread");
                                break 'global;
                            }
                        },
                    }
                }
            }
        }
        drop(_span);
    }

    // If we get to here, it's time to exit the thread and shutdown
    info!("engine thread exiting");

    trace!("unsubscribing message receiver");
    message_receiver.unsubscribe();
    trace!("unsubscribing message sender");
    message_sender.unsubscribe();

    trace!("dropping {}", name_of!(_no_panic_pill));
    drop(_no_panic_pill);
    return;
}
