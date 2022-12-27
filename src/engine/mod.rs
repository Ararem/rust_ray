use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::Duration;

use crate::FallibleFn;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use nameof::name_of;
use tracing::{debug, debug_span, info_span, trace, trace_span};
use crate::config::Config;

use crate::helper::logging::event_targets::*;
use crate::program::program_data::ProgramData;
use crate::program::thread_messages::ThreadMessage::{Engine, Program, Ui};
use crate::program::thread_messages::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EngineData {}

pub(crate) fn engine_thread(
    thread_start_barrier: Arc<Barrier>,
    _program_data_wrapped: Arc<Mutex<ProgramData>>,
    message_sender: BroadcastSender<ThreadMessage>,
    message_receiver: BroadcastReceiver<ThreadMessage>,
    config: Config
) -> FallibleFn {
    let span_engine_thread =
        info_span!(target: THREAD_DEBUG_GENERAL, parent: None, "engine_thread").entered();

    {
        let span_sync_thread_start =
            debug_span!(target: THREAD_DEBUG_GENERAL, "sync_thread_start").entered();
        trace!(
            target: THREAD_DEBUG_GENERAL,
            "waiting for {}",
            name_of!(thread_start_barrier)
        );
        thread_start_barrier.wait();
        trace!(
            target: THREAD_DEBUG_GENERAL,
            "wait complete, running engine thread"
        );
        span_sync_thread_start.exit();
    }

    let span_global_loop = debug_span!(target: ENGINE_TRACE_GLOBAL_LOOP, "'global").entered();
    'global: for global_iter in 0usize.. {
        let span_global_loop_inner =
            trace_span!(target: ENGINE_TRACE_GLOBAL_LOOP, "inner", global_iter).entered();

        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        let span_process_messages =
            trace_span!(target: THREAD_TRACE_MESSAGE_LOOP, "process_messages").entered();
        // Loops until [command_receiver] is empty (tries to 'flush' out all messages)
        'process_messages: loop {
            if let Some(message) = receive_message(&message_receiver)? {
                match message {
                    Ui(_) | Program(_) => {
                        message.ignore();
                        continue 'process_messages;
                    }
                    Engine(engine_message) => {
                        debug!(
                            target: THREAD_DEBUG_MESSAGE_RECEIVED,
                            ?engine_message,
                            "got engine message"
                        );
                        match engine_message {
                            EngineThreadMessage::ExitEngineThread => {
                                debug!(
                                    target: THREAD_DEBUG_GENERAL,
                                    "got exit message for engine thread"
                                );
                                break 'global;
                            }
                        }
                    }
                }
            }
            // No messages waiting
            else {
                break 'process_messages;
            }
        }
        span_process_messages.exit();

        span_global_loop_inner.exit();
    }
    span_global_loop.exit();

    // If we get to here, it's time to exit the thread and shutdown
    debug!(target: THREAD_DEBUG_GENERAL, "engine thread exiting");

    debug!(
        target: THREAD_DEBUG_MESSENGER_LIFETIME,
        "unsubscribing message receiver"
    );
    message_receiver.unsubscribe();
    debug!(
        target: THREAD_DEBUG_MESSENGER_LIFETIME,
        "unsubscribing message sender"
    );
    message_sender.unsubscribe();

    debug!(target: THREAD_DEBUG_GENERAL, "engine thread done");
    span_engine_thread.exit();
    Ok(())
}
