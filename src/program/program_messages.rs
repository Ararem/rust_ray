//! Internal module that contains implementations of enums for messages that can be sent upstream by the engine and UI threads to the main thread

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