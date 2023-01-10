use crate::helper::logging::event_targets::*;
use crate::FallibleFn;
use imgui::Ui;
use tracing::{debug, trace, trace_span};

pub fn menu<T: FnOnce() -> FallibleFn>(ui: &Ui, name: &str, generate_menu_items: T) -> FallibleFn {
    trace_span!(target: UI_TRACE_BUILD_INTERFACE, "tools_menu").in_scope(|| {
        let menu_token = match ui.begin_menu(name) {
            None => {
                trace!(target: UI_TRACE_USER_INPUT, name, selected = false);
                trace!(
                    target: UI_TRACE_BUILD_INTERFACE,
                    "{} menu not visible",
                    name
                );
                return Ok(());
            }
            Some(token) => {
                trace!(target: UI_TRACE_USER_INPUT, name, selected = true);
                trace!(target: UI_TRACE_BUILD_INTERFACE, "{} menu visible", name);
                token
            }
        };
        let result = generate_menu_items();
        menu_token.end();
        result
    })
}

/// Creates a toggle menu item for a mutable bool reference
pub fn toggle_menu_item(
    ui: &Ui,
    name: &str,
    toggle: &mut bool,
    shortcut_text: &str,
    tooltip: &str,
) -> FallibleFn {
    let span_toggle_menu_item = trace_span!(
        target: UI_TRACE_BUILD_INTERFACE,
        "toggle_menu_item",
        toggle_name = name,
    )
    .entered();
    trace!(
        target: UI_TRACE_BUILD_INTERFACE,
        toggle_value = toggle,
        shortcut_text,
        tooltip
    );

    let clicked = ui
        .menu_item_config(name)
        .shortcut(shortcut_text)
        .build_with_ref(toggle);
    trace!(target: UI_TRACE_USER_INPUT, name, clicked);
    if clicked {
        // Don't need to toggle manually since it's handled by ImGui (we passed in a mut ref to the variable)
        debug!(
            target: UI_DEBUG_USER_INTERACTION,
            "clicked menu item '{}', value: {}", name, *toggle
        );
    }

    span_toggle_menu_item.exit();
    Ok(())
}
