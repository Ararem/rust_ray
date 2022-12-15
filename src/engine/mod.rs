use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use color_eyre::eyre;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug_span, info, info_span, instrument, trace};
use crate::helper::logging::event_targets::THREAD_DEBUG_GENERAL;

use crate::program::thread_messages::ThreadMessage::{Engine, Program, Ui};
use crate::program::thread_messages::{
    unreachable_never_should_be_disconnected, EngineThreadMessage, ThreadMessage,
};
use crate::program::program_data::ProgramData;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EngineData {}

pub(crate) fn engine_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<ThreadMessage>,
    message_receiver: BroadcastReceiver<ThreadMessage>,
) -> eyre::Result<()> {
    let _span = info_span!(target: THREAD_DEBUG_GENERAL, "engine_thread");

    {
        let _span = debug_span!(target: THREAD_DEBUG_GENERAL, "sync_thread_start");
        trace!(target: THREAD_DEBUG_GENERAL,"waiting for {}", name_of!(thread_start_barrier));
        thread_start_barrier.wait();
        trace!(target: THREAD_DEBUG_GENERAL, "wait complete, running engine thread");
    }
    'global: loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        let _span = debug_span!("process_messages").entered();
        'loop_messages: loop {
            // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
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
                        Ui(_ui_message) => {
                            trace!(
                                target: THREAD_MESSAGE_PROCESSING_SPAMMY,
                                "[engine] message for ui thread, ignoring"
                            );
                            continue 'loop_messages;
                        }
                        Program(_program_message) => {
                            trace!(
                                target: THREAD_MESSAGE_PROCESSING_SPAMMY,
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

    Ok(())
}
