use std::any::type_name;
use std::error::Error;
use std::fmt::{Debug, Formatter};

use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::ClipboardBackend;
use tracing::*;

/// Wrapper struct for [ClipboardContext] that allows integration with [imgui]
/// Used to implement [ClipboardBackend]
pub(in crate::ui) struct ImguiClipboardSupport {
    /// The wrapped [ClipboardContext] object that the operations are passed to
    backing_context: ClipboardContext,
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
#[instrument(ret, level = "trace")]
pub(in crate::ui) fn clipboard_init() -> Result<ImguiClipboardSupport, Box<dyn Error>> {
    ClipboardContext::new().map(|val| ImguiClipboardSupport {
        backing_context: val,
    })
}

impl ClipboardBackend for ImguiClipboardSupport {
    fn get(&mut self) -> Option<String> {
        let contents = self.backing_context.get_contents().ok();
        trace!("got clipboard: {contents:?}");
        contents
    }
    fn set(&mut self, text: &str) {
        let result = self.backing_context.set_contents(text.to_owned());
        if let Err(error) = result {
            warn!("could not set clipboard due to error: {error}")
        } else {
            trace!("set clipboard: {text:?}");
        }
    }
}
