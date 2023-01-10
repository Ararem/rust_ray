//! Internal module that contains implementations of enums for messages that can be sent upstream by the engine and UI threads to the main thread

use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError::{Disconnected, Full};
use std::sync::Arc;

use color_eyre::{eyre, Help, Report, SectionExt};
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use tracing::{debug, trace};

use crate::FallibleFn;
use ThreadMessage::{Engine, Program, Ui};

use crate::helper::logging::event_targets::*;

/// Base message struct, contains variants for which thread it should be targeted at
#[derive(Debug, Clone)]
pub(crate) enum ThreadMessage {
    Engine(EngineThreadMessage),
    Program(ProgramThreadMessage),
    Ui(UiThreadMessage),
}

// ========== PROGRAM THREAD ==========

#[derive(Debug, Clone)]
pub(crate) enum ProgramThreadMessage {
    /// The app should quit, but gently (not due to an error, like the user hit the quit button)
    QuitAppNoError(QuitAppNoErrorReason),
    /// The app should quit, because an error happened
    ///
    /// # Notes:
    /// Uses an [Arc<T>] to wrap the report because we can't clone a [Report].
    /// We need to be able to clone because that's required by [multiqueue2]
    QuitAppError(Arc<Report>),
}

/// Reasons why the app should quit, but not because of an error (a good quit)
#[derive(Debug, Clone, Copy)]
pub(crate) enum QuitAppNoErrorReason {
    /// The user made an interaction that means the app should quit
    QuitInteractionByUser,
}

// ========== UI THREAD ==========

/// A message that will be read by the UI thread
#[derive(Debug, Clone, Copy)]
pub(crate) enum UiThreadMessage {
    /// The UI thread should exit
    ExitUiThread,
}

// ========== ENGINE THREAD ==========

/// A message that will be read by the engine thread, and acted upon according to it's contents
#[derive(Debug, Clone, Copy)]
pub(crate) enum EngineThreadMessage {
    /// The engine thread should exit
    ExitEngineThread,
}

// ========== MACROS AND FUNCTIONS ==========

impl ThreadMessage {
    /// Consumes a [ThreadMessage], marking it as ignored
    ///
    /// Also logs a message that it was ignored
    pub(crate) fn ignore(self) {
        let target_thread = match self {
            Engine(_) => "engine",
            Program(_) => "program",
            Ui(_) => "ui",
        };
        trace!(target: THREAD_TRACE_MESSAGE_IGNORED, ?self, "ignoring message for {}", target_thread);
    }
}

/// Function that contains shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_recv] returns [std::sync::mpsc::TryRecvError::Disconnected] in any of the message loops
///
/// # ***THIS IS BAD:***
/// Should (only) get here once all senders have disconnected
///
/// However, this should only happen when the threads are told to quit, after which the main function quits...
///
/// Also, the main thread sender should never be dropped
///
/// So (in working code) we should never get here
pub(crate) fn error_recv_never_should_be_disconnected() -> Report {
    Report::msg(indoc::formatdoc! {
        "all message channel senders were dropped: [try_recv()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TryRecvError::Disconnected
    })
    .wrap_err("critical invalid state")
    .note(indoc::formatdoc! {r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else
    "})
}
/// Function that contains shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_send] returns [std::sync::mpsc::TrySendError::Disconnected] in any of the message loops
///
/// # ***THIS IS BAD:***
/// Should (only) get here once all receivers have disconnected
///
/// However, this should only happen when the threads are told to quit, after which the main function quits...
///
/// Also, the main thread receiver should never be dropped until shutdown
///
/// So (in working code) we should never get here
pub(crate) fn error_send_never_should_be_disconnected() -> Report {
    Report::msg(indoc::formatdoc! {
        "all message channel senders were dropped: [try_recv()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TryRecvError::Disconnected
    })
    .wrap_err("critical invalid state")
    .note(indoc::formatdoc! {r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else
    "})
}

/// Function that contains shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_send] returns [std::sync::mpsc::TrySendError::Full] in any of the message loops
///
/// # ***THIS IS BAD:***
/// The receivers should always be receiving and reading messages as long as they are connected
///
/// If we get here, either we have (massively) overloaded the channel (shouldn't be possible, we have a high limit),
/// Or, one of the threads is deadlocked/stuck somewhere
///
/// Either way, shouldn't be possible, very bad
pub(crate) fn error_never_should_be_full() -> Report {
    Report::msg(indoc::formatdoc! {
        "message channel was full: [try_send()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TrySendError::Full(0) //argument is arbitrary
    })
    .wrap_err("critical invalid state")
    .note(indoc::formatdoc! {r"
        message buffer should never be full - it has a high capacity and receivers should constantly be polling.
        most likely, one of the other threads crashed or deadlocked (and therefore can't receive)
    "})
}

/// Receives a [ThreadMessage] from a [BroadcastReceiver]
///
/// # Return
/// `Ok(None)` => No messages waiting
/// `Ok(Some(<message>))` => received a message
/// `Err(<error>)` => Something bad happened. This cause should be considered fatal
pub(crate) fn receive_message(receiver: &BroadcastReceiver<ThreadMessage>) -> eyre::Result<Option<ThreadMessage>> {
    trace!(target: THREAD_TRACE_MESSAGE_LOOP, "message_receiver.try_recv()");
    let maybe_message = receiver.try_recv();
    trace!(target: THREAD_TRACE_MESSAGE_LOOP, ?maybe_message);
    match maybe_message {
        Err(TryRecvError::Empty) => {
            trace!(target: THREAD_TRACE_MESSAGE_LOOP, "no messages (Err::Empty)");
            Ok(None) // Exit the message loop, go into waiting
        }
        Err(TryRecvError::Disconnected) => Err(error_recv_never_should_be_disconnected()),
        Ok(message) => {
            trace!(target: THREAD_TRACE_MESSAGE_LOOP, ?message, "got message");
            Ok(Some(message))
        }
    }
}

pub(crate) fn send_message(message: ThreadMessage, sender: &BroadcastSender<ThreadMessage>) -> FallibleFn {
    debug!(target: THREAD_DEBUG_MESSAGE_SEND, ?message);
    match sender.try_send(message) {
        Ok(()) => Ok(()),

        // Neither of these errors should happen ever, but better to be safe
        Err(Disconnected(_failed_message)) => {
            return Err(error_recv_never_should_be_disconnected().section(format!("{_failed_message:?}").header("Message")));
        }
        Err(Full(_failed_message)) => {
            return Err(error_never_should_be_full().section(format!("{_failed_message:?}").header("Message")));
        }
    }
}
