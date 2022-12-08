use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug_span, info, instrument, trace};

use crate::helper::logging::event_targets::*;
use crate::program::program_messages::{Message, UiThreadMessage};
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::ProgramData;
use crate::program_thread_messaging__unreachable_never_should_be_disconnected;

#[derive(Copy, Clone, Debug)]
pub struct UiData {}

#[instrument(ret, skip_all)]
pub(crate) fn ui_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<Message>,
    message_receiver: BroadcastReceiver<Message>,
) {
    //Create a NoPanicPill to make sure we exit if anything panics
    let _no_panic_pill = crate::helper::panic_pill::PanicPill {};

    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running ui thread");

    'outer: loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        let _span = debug_span!("process_messages").entered();
        'loop_messages: loop {
            // Loops until [message_receiver] is empty (tries to 'flush' out all messages)
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
                                break 'outer;
                            },
                        },
                    }
                }
            }
        }
        drop(_span);
    }

    // If we get to here, it's time to exit the thread and shutdown
    info!("ui thread exiting");

    trace!("unsubscribing message receiver");
    message_receiver.unsubscribe();
    trace!("unsubscribing message sender");
    message_sender.unsubscribe();

    trace!("dropping {}", name_of!(_no_panic_pill));
    drop(_no_panic_pill);
    return;
}
