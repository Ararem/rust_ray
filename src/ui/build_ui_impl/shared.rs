//! Module of shared functions used for the UI building
use crate::config::read_config_value;
use crate::config::run_time::keybindings_config::KeyBinding;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_error_string_no_ansi;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::{Handler, Report};
use imgui::{Condition, StyleColor, Ui};
use itertools::Itertools;
use tracing::{debug, trace, trace_span};
use tracing_error::ExtractSpanTrace;
use tracing_subscriber::fmt::format;

/*
===================
===== WINDOWS =====
===================
*/
pub fn build_window<T: UiItem>(
    label: &str,
    item: &mut T,
    opened: &mut bool,
    ui: &Ui,
) -> FallibleFn {
    let span_window = trace_span!(
        target: UI_TRACE_BUILD_INTERFACE,
        "build_window",
        window = label
    )
    .entered();
    let mut result = Ok(());
    if *opened {
        let token = ui
            .window(label)
            .opened(opened)
            .size([300.0, 110.0], Condition::FirstUseEver)
            .begin();
        if let Some(token) = token {
            result = item.render(ui, true);
            token.end();
        } else {
            result = item.render(ui, false)
        }
    }
    span_window.exit();
    result
}
pub fn build_window_fn(
    label: &str,
    func: fn(&Ui, bool) -> FallibleFn,
    opened: &mut bool,
    ui: &Ui,
) -> FallibleFn {
    let span_window = trace_span!(
        target: UI_TRACE_BUILD_INTERFACE,
        "build_window",
        window = label
    )
    .entered();
    let mut result = Ok(());
    if *opened {
        let token = ui
            .window(label)
            .opened(opened)
            .size([300.0, 110.0], Condition::FirstUseEver)
            .begin();
        if let Some(token) = token {
            result = func(ui, true);
            token.end();
        } else {
            result = func(ui, false)
        }
    }
    span_window.exit();
    result
}

/*
=====================
===== MENU BARS =====
=====================
 */

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

/*
=================
===== INPUT =====
=================
*/

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

/*
===================
===== WIDGETS =====
===================
*/

pub fn display_eyre_report(ui: &Ui, report: &Report) {
    let colours = read_config_value(|config| config.runtime.ui.colours);
    ui.text("Chain");
    let col = ui.push_style_color(StyleColor::Text, [0.3, 0.5, 1.0, 1.0]);
    for err in report.chain() {
        ui.bullet_text(format!("{}", err))
    }
    col.end();
    ui.text(format!("Source:\n{:#?}", report.source()));
    ui.text(format!(
        "Backtrace:\n{:#?}",
        report
            .handler()
            .downcast_ref::<color_eyre::Handler>()
            .map(|h| h.backtrace())
    ));
    ui.text(format!(
        "Span Trace:\n{:#?}",
        report
            .handler()
            .downcast_ref::<color_eyre::Handler>()
            .map(|h| h.span_trace())
    ));
}
