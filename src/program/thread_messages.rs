//! Internal module that contains implementations of enums for messages that can be sent upstream by the engine and UI threads to the main thread

use std::sync::Arc;

use color_eyre::{Help, Report};
use tracing::{trace};

use ThreadMessage::{Engine, Program, Ui};

use crate::helper::logging::event_targets::THREAD_TRACE_MESSAGE_IGNORED;

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
        trace!(
            target: THREAD_TRACE_MESSAGE_IGNORED,
            ?self,
            "ignoring message for {}",
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
/// Also, the main thread sender should never be dropped
///
/// So (in working code) we should never get here
pub(crate) fn error_recv_never_should_be_disconnected() -> Report {
    let report = Report::msg(indoc::formatdoc! {
        "all message channel senders were dropped: [try_recv()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TryRecvError::Disconnected
    }).wrap_err("critical invalid state")
      .note(indoc::formatdoc! {r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else
    "});
    report
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
    let report = Report::msg(indoc::formatdoc! {
        "all message channel senders were dropped: [try_recv()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TryRecvError::Disconnected
    }).wrap_err("critical invalid state")
      .note(indoc::formatdoc! {r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else
    "});
    report
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
    let report = Report::msg(indoc::formatdoc! {
        "message channel was full: [try_send()] returned [{0:?}] \"{0}\"",
        ::std::sync::mpsc::TrySendError::Full(0) //argument is arbitrary
    }).wrap_err("critical invalid state").note(
        indoc::formatdoc! {r"
        message buffer should never be full - it has a high capacity and receivers should constantly be polling.
        most likely, one of the other threads crashed or deadlocked (and therefore can't receive)
    "}
    );
    report
}
