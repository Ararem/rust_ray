use crate::config::run_time::keybindings_config::KeyBinding;
use crate::helper::logging::event_targets::*;
use imgui::Ui;
use tracing::{debug, trace, trace_span};

pub fn handle_shortcut(ui: &Ui, name: &str, keybind: &KeyBinding, toggle: &mut bool) {
    trace_span!(target: UI_TRACE_USER_INPUT, "handle_shortcut", name, %keybind).in_scope(||{
        let key_pressed = ui.is_key_index_pressed_no_repeat(keybind.shortcut as i32);
        let modifiers_pressed = keybind.required_modifiers_held(ui);
        trace!(target: UI_TRACE_USER_INPUT, key_pressed, modifiers_pressed);
        if key_pressed && modifiers_pressed{
            *toggle ^= true;
            debug!(target: UI_DEBUG_USER_INTERACTION, %keybind, "keybind for {} pressed, value: {}", name, toggle)
        }
    });
}
