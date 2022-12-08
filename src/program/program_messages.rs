//! Internal module that contains implementations of enums for messages that can be sent upstream by the engine and UI threads to the main thread
#![macro_use]

use std::sync::Arc;

use color_eyre::Report;

/// Base message struct, contains variants for which thread it should be targeted at
#[derive(Debug, Clone)]
pub(crate) enum Message {
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
    /// Uses an [Arc<T>] to wrap the report because we can't clone a [Report]
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

// ========== MACROS ==========

/// Macro that injects shared code for the case when [multiqueue2::broadcast::BroadcastReceiver::try_recv] returns [std::sync::mpsc::TryRecvError::Disconnected] in any of the message loops
///
/// # ***THIS IS BAD:***
/// Should (only) get here once all senders have disconnected
///
/// However, this should only happen when the threads are told to quit, after which the main function quits...
///
/// Also, the main (this) thread sender should never be dropped
///
/// So (in working code) we should never get here
#[macro_export]
macro_rules! program_thread_messaging__unreachable_never_should_be_disconnected {
    () => {{
            let message_heading =::indoc::formatdoc!(r#"
            all message channel senders were dropped: [try_recv()] returned [{0:?}] "{0}""#
                , ::std::sync::mpsc::TryRecvError::Disconnected);
            let message_body = ::indoc::formatdoc!(r"
            ui/engine senders should only be dropped when exiting threads, and program sender should never be dropped.
            something probably went (badly) wrong somewhere else");
            unreachable!("{}\n\n{}", message_heading, message_body);
            // return Err(Report::msg(message_heading).note(message_body));
    }};
}