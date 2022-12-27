//! Support module that allows for using the clipboard in [imgui]
use std::any::type_name;
use std::fmt::{Debug, Formatter};

use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::{eyre, Help, SectionExt};
use imgui::ClipboardBackend;
use tracing::*;
use crate::config::Config;

use crate::helper::logging::event_targets::{GENERAL_WARNING_NON_FATAL, UI_DEBUG_USER_INTERACTION};
use crate::helper::logging::{dyn_error_to_report, format_error};

/// Wrapper struct for [ClipboardContext] that allows integration with [imgui]
/// Used to implement [ClipboardBackend]
pub(in crate::ui) struct ImguiClipboardSupport {
    /// The wrapped [ClipboardContext] object that the operations are passed to
    backing_context: ClipboardContext,
    config: Config
}

impl Debug for ImguiClipboardSupport {
    /// [Debug] implementation for [ClipboardContext].
    ///
    /// Since the [ClipboardContext] type is just an alias, and it exposes no internals, this simply returns the name of the type (using [type_name])
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<ClipboardContext>())
    }
}

/// (Tries to) initialise clipboard support
pub(in crate::ui) fn clipboard_init(config: Config) -> eyre::Result<ImguiClipboardSupport> {
    match ClipboardContext::new() {
        Ok(val) => Ok(ImguiClipboardSupport {
            backing_context: val,
            config
        }),
        Err(boxed_error) => {
            let report =
                dyn_error_to_report(&boxed_error, config).wrap_err("could not get clipboard context");
            Err(report)
        }
    }
}

impl ClipboardBackend for ImguiClipboardSupport {
    fn get(&mut self) -> Option<String> {
        let span_get_clipboard =
            debug_span!(target: UI_DEBUG_USER_INTERACTION, "get_clipboard").entered(); //Dropped on return
        let get_result = self.backing_context.get_contents();
        debug!(target: UI_DEBUG_USER_INTERACTION, ?get_result);
        let maybe_text = match get_result {
            Err(boxed_error) => {
                let report =
                    dyn_error_to_report(&boxed_error,self.config).wrap_err("could not get clipboard text");
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    error = format_error(&report,self.config),
                    "couldn't get clipboard"
                );
                None
            }
            Ok(clipboard_text) => {
                trace!(
                    target: UI_DEBUG_USER_INTERACTION,
                    clipboard_text,
                    "got clipboard"
                );
                Some(clipboard_text)
            }
        };
        span_get_clipboard.exit();
        maybe_text
    }

    fn set(&mut self, clipboard_text: &str) {
        let span_set_clipboard = debug_span!(
            target: UI_DEBUG_USER_INTERACTION,
            "set_clipboard",
            clipboard_text
        )
        .entered();
        let set_result = self.backing_context.set_contents(clipboard_text.to_owned());
        debug!(target: UI_DEBUG_USER_INTERACTION, ?set_result);
        if let Err(boxed_error) = set_result {
            let report = dyn_error_to_report(&boxed_error, self.config)
                .wrap_err("could not set clipboard text")
                .section(clipboard_text.to_owned().header("Clipboard:"));
            warn!(
                target: GENERAL_WARNING_NON_FATAL,
                error = format_error(&report,self.config),
                "couldn't set clipboard"
            )
        } else {
            trace!(
                target: UI_DEBUG_USER_INTERACTION,
                clipboard_text = clipboard_text,
                "set clipboard"
            );
        }
        span_set_clipboard.exit();
    }
}
