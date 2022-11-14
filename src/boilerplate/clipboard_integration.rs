use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::ClipboardBackend;
use std::error::Error;
use tracing::*;

/// Wrapper struct for [ClipboardContext] that allows integration with [imgui]
/// Used to implement [ClipboardBackend]
pub struct ImguiClipboardSupport {
    /// The wrapped [ClipboardContext] object that the operations are passed to
    backing_context: ClipboardContext,
}

/// (Tries to) initialise clipboard support
pub fn init() -> Result<ImguiClipboardSupport, Box<dyn Error>> {
    ClipboardContext::new().map(|val| ImguiClipboardSupport {
        backing_context: val,
    })
}

impl ClipboardBackend for ImguiClipboardSupport {
    fn get(&mut self) -> Option<String> {
        let contents = self.backing_context.get_contents().ok();
        trace!("Clipboard is {contents:?}");
        contents
    }
    fn set(&mut self, text: &str) {
        // ignore errors?
        //TODO: Don't ignore clipboard set errors
        let result = self.backing_context.set_contents(text.to_owned());
        trace!("Clipboard set to {text:?}");
    }
}
