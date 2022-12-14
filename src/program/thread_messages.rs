//! Internal module that contains implementations of enums for messages that can be sent upstream by the engine and UI threads to the main thread

use std::sync::Arc;

use color_eyre::Report;
use tracing::{debug, trace};
use ThreadMessage::{Engine, Program, Ui};
use crate::helper::logging::event_targets::{THREAD_DEBUG_MESSAGE_RECEIVED, THREAD_TRACE_MESSAGE_IGNORED};

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
#[derive(Debug, Clone)]
pub(crate) enum QuitAppNoErrorReason {
    /// The user made an interaction that means the app should quit
    QuitInteractionByUser,
}

// ========== UI THREAD ==========

/// A message that will be read by the UI thread
#[derive(Debug, Clone)]
pub(crate) enum UiThreadMessage {
    /// The UI thread should exit
    ExitUiThread,
}

// ========== ENGINE THREAD ==========

/// A message that will be read by the engine thread, and acted upon according to it's contents
#[derive(Debug, Clone)]
pub(crate) enum EngineThreadMessage {
    /// The engine thread should exit
    ExitEngineThread,
}

// ========== MACROS AND FUNCTIONS ==========

impl ThreadMessage {
    pub(crate) fn log_ignored(&self) {
        let target_thread = match self {
            Engine(_) => "engine",
            Program(_) => "program",
            Ui(_) => "ui",
        };
        trace!(
            target: THREAD_TRACE_MESSAGE_IGNORED,
            ?self,
            "ignoring message for {}"
            target_thread
        );
    }
    pub(crate) fn log_received(&self) {
        let target_thread = match self {
            Engine(_) => "engine",
            Program(_) => "program",
            Ui(_) => "ui",
        };
        debug!(
            target: THREAD_DEBUG_MESSAGE_RECEIVED,
            ?self,
            "got {} message",
            target_thread
        );
    }
}


/// Function that contains shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_recv] returns [std::sync::mpsc::TryRecvError::Disconnected] in any of the message loops
///
/// # ***THIS IS BAD:***
/// Should (only) get here once all senders have disconnected
///
/// However, this should only happen when the threads are told to quit, after which the main function quits...
///
/// Also, the main (this) thread sender should never be dropped
///
/// So (in working code) we should never get here
pub(crate) fn unreachable_never_should_be_disconnected() -> ! {
    let message_heading = indoc::formatdoc!(
        r#"
            all message channel senders were dropped: [try_recv()] returned [{0:?}] "{0}""#,
        ::std::sync::mpsc::TryRecvError::Disconnected
    );
    let message_body = indoc::formatdoc!(
        r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else"
    );
    panic!("invalid state: {}\n\n{}", message_heading, message_body);
}

/// Function that contains shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_recv] returns [std::sync::mpsc::TryRecvError::Full] in any of the message loops
///
/// # ***THIS IS BAD:***
/// The receivers should always be receiving and reading messages as long as they are connected
///
/// If we get here, either we have (massively) overloaded the channel (shouldn't be possible, we have a high limit),
/// Or, one of the threads is deadlocked/stuck somewhere
///
/// Either way, shouldn't be possible, will cause panic
pub(crate) fn unreachable_never_should_be_full() -> ! {
    let message_heading = indoc::formatdoc!(
        r#"
            all message channel senders were dropped: [try_recv()] returned [{0:?}] "{0}""#,
        ::std::sync::mpsc::TryRecvError::Disconnected
    );
    let message_body = indoc::formatdoc!(
        r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else"
    );
    panic!("invalid state: {}\n\n{}", message_heading, message_body);
}
