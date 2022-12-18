use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use color_eyre::eyre;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug, debug_span, info_span, trace, trace_span};

use crate::helper::logging::event_targets::*;
use crate::program::program_data::ProgramData;
use crate::program::thread_messages::{
    EngineThreadMessage, ThreadMessage, unreachable_never_should_be_disconnected,
};
use crate::program::thread_messages::ThreadMessage::{Engine, Program, Ui};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EngineData {}

pub(crate) fn engine_thread(
    thread_start_barrier: Arc<Barrier>,
    program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<ThreadMessage>,
    message_receiver: BroadcastReceiver<ThreadMessage>,
) -> eyre::Result<()> {
    let _engine_thread_span = info_span!(target: THREAD_DEBUG_GENERAL, "engine_thread");

    {
        let _span = debug_span!(target: THREAD_DEBUG_GENERAL, "sync_thread_start");
        trace!(target: THREAD_DEBUG_GENERAL,"waiting for {}", name_of!(thread_start_barrier));
        thread_start_barrier.wait();
        trace!(target: THREAD_DEBUG_GENERAL, "wait complete, running engine thread");
    }

    let span_global_loop = debug_span!(target: ENGINE_TRACE_GLOBAL_LOOP, "'global");
    'global: for global_iter in 0usize.. {
        let span_global_loop_inner = trace_span!(target: ENGINE_TRACE_GLOBAL_LOOP, "inner", global_iter);

        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        let span_process_messages = trace_span!(target: THREAD_TRACE_MESSAGE_LOOP,"process_messages").entered();
        // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
        'process_messages: loop {
            trace!(
                target: THREAD_TRACE_MESSAGE_LOOP,
                "message_receiver.try_recv()"
            );
            let maybe_message = message_receiver.try_recv();
            trace!(target: THREAD_TRACE_MESSAGE_LOOP, ?maybe_message);
            match maybe_message {
                Err(TryRecvError::Empty) => {
                    trace!(
                        target: THREAD_TRACE_MESSAGE_LOOP,
                        "no messages (Err::Empty)"
                    );
                    break 'process_messages; // Exit the message loop, go into waiting
                }
                Err(TryRecvError::Disconnected) => {
                    unreachable_never_should_be_disconnected();
                }
                Ok(message) => {
                    trace!(
                        target: THREAD_TRACE_MESSAGE_LOOP,
                        ?message,
                        "got message"
                    );
                    match message {
                        Ui(_) | Program(_) => {
                            message.log_ignored();
                            continue 'process_messages;
                        }
                        Engine(engine_message) => {
                            message.log_received();
                            match engine_message {
                                EngineThreadMessage::ExitEngineThread => {
                                    debug!(target: THREAD_DEBUG_GENERAL, "got exit message for engine thread");
                                    break 'global;
                                }
                            }
                        },
                    }
                }
            }
        }
        drop(span_process_messages);

        drop(span_global_loop_inner);
    }
    drop(span_global_loop);

    // If we get to here, it's time to exit the thread and shutdown
    debug!(target: THREAD_DEBUG_GENERAL, "engine thread exiting");

    debug!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "unsubscribing message receiver");
    message_receiver.unsubscribe();
    debug!(target: THREAD_DEBUG_MESSENGER_LIFETIME, "unsubscribing message sender");
    message_sender.unsubscribe();

    debug!(target: THREAD_DEBUG_GENERAL, "engine thread done");
    Ok(())
}
