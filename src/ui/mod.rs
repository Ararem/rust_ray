use std::io::stdin;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use flume::{Receiver, Sender, TryRecvError};
use flume::TryRecvError::Disconnected;
use nameof::name_of;
use tracing::{info, instrument, trace};
use TryRecvError::Empty;

use crate::helper::logging::event_targets::*;
use crate::program::program_messages::{Message, QuitAppNoErrorReason, UiThreadMessage};
use crate::program::program_messages::Message::{Engine, Program, Ui};
use crate::program::program_messages::ProgramThreadMessage::QuitAppNoError;
use crate::program::ProgramData;

#[derive(Copy, Clone, Debug)]
pub struct UiData {}

#[instrument(ret, skip_all)]
pub(crate) fn ui_thread(
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

        let mut s = String::new();
        if let Ok(_) = stdin().read_line(&mut s) {
            info!("sending quit");
            message_sender.send(Message::Program(QuitAppNoError(
                QuitAppNoErrorReason::QuitInteractionByUser,
            ))).expect("dev code, expect message to send");
        }

        'loop_messages: loop {
            // Loops until [message_receiver] is empty (tries to 'flush' out all messages)
            let recv = message_receiver.try_recv();
            match recv {
                Err(Empty) => {
                    trace!(target: PROGRAM_MESSAGE_POLL_SPAMMY, "no messages waiting");
                    break 'loop_messages; // Exit the message loop, go into waiting
                }
                Err(Disconnected) => {
                    // Should (only) get here once the program and engine threads have exited, and therefore they have dropped their sender variables
                    // This is not supposed to happen, since the program thread should always be the last to exit
                    unreachable!(r"ui thread {} returned {}, which shouldn't be possible (the program thread should always be alive while the UI thread is alive)", name_of!(message_receiver), Disconnected);
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
                                info!("got exit message for Ui thread");
                                return;
                            },
                        },
                    }
                }
            }
        }
    }

    drop(_no_panic_pill);
}
