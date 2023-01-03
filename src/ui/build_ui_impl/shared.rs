//! Module of shared functions used for the UI building
use crate::config::read_config_value;
use crate::config::run_time::keybindings_config::KeyBinding;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_string_no_ansi;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::Report;
use imgui::{Condition, StyleColor, Ui};
use itertools::Itertools;
use tracing::{debug, trace, trace_span, Metadata};
use tracing_error::SpanTraceStatus;

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
    let span_display_error_report =
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "display_error_report").entered();
    let colours = read_config_value(|config| config.runtime.ui.colours);
    macro_rules! section {
        ($title:literal, $body:block) => {
        let span_section = trace_span!(target: UI_TRACE_BUILD_INTERFACE, $title).entered();
        let maybe_node = ui.tree_node_config($title).opened(true, Condition::FirstUseEver).push(); // Should be open by default
        if let Some(opened_node) = maybe_node{
            trace!(target: UI_TRACE_BUILD_INTERFACE, "node expanded");
            $body
            opened_node.end();
        }
        else{
            trace!(target: UI_TRACE_BUILD_INTERFACE, "node closed");
        }

        span_section.exit();
        };
    }
    section!("Chain", {
        for err in report.chain() {
            // We don't use the alternate specifier since we just want the single error, not sub-errors
            let err_string = err.to_string();
            trace!(target: UI_TRACE_BUILD_INTERFACE, "[Bullet] {}", err_string);
            ui.bullet();
            ui.same_line();
            ui.text_colored(colours.value.error_message, err_string);
        }
    });
    section!("Backtrace", {
        if let Some(handler) = report.handler().downcast_ref::<color_eyre::Handler>() {
            if let Some(backtrace) = handler.backtrace() {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "have backtrace");
                ui.text("TODO: Backtrace display");
            } else {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "missing backtrace");
                ui.text_colored(colours.severity.warning, "This error doesn't have a backtrace. Try checking `RUST_BACKTRACE` and/or `RUST_BACKTRACE` environment variables are set")
            }
        } else {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "couldn't cast handler");
            ui.text_colored(
                colours.severity.warning,
                "Couldn't downcast error report's handler to get the backtrace",
            );
        }
    });
    section!("Span trace", {
        if let Some(handler) = report.handler().downcast_ref::<color_eyre::Handler>() {
            if let Some(span_trace) = handler.span_trace() {
                match span_trace.status() {
                    SpanTraceStatus::UNSUPPORTED => {
                        trace!(
                            target: UI_TRACE_BUILD_INTERFACE,
                            "span trace: not supported"
                        );
                        ui.text_colored(colours.severity.warning, "SpanTraces are not supported, likely because there is no [ErrorLayer] or the [ErrorLayer] is from a different version of [tracing_error]")
                    }
                    SpanTraceStatus::EMPTY => {
                        trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: empty");
                        ui.text_colored(colours.severity.warning, "The SpanTrace is empty, likely because it was captured outside of any spans")
                    }
                    SpanTraceStatus::CAPTURED => {
                        trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: captured");
                        // [with_spans] calls the closure on every span in the trace, as long as the closure returns `true`
                        let mut depth = 0;
                        span_trace.with_spans(
                            |metadata: &'static Metadata<'static>, formatted_span_fields: &str| -> bool {
                                let span_process_span = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "process_span", ?metadata, formatted_span_fields=formatted_span_fields.to_owned()).entered();
                                // Construct a tree node with the span name as the title
                                // If the node is expanded, then we get to see all the juicy information
                                let tree_node_colour_style = ui.push_style_color(StyleColor::Text, colours.value.tracing_event_name); //Colour the title
                                let maybe_node = ui.tree_node(format!("{depth}: {}", metadata.name()));
                                tree_node_colour_style.pop();

                                if let Some(node) = maybe_node{
                                    const ALIGN: usize = 12;
                                    macro_rules! metadata_label {
                                        ($name:expr) => {{
                                            // Label for what piece of metadata it is
                                            // It includes a colon, and left-aligns the entire string (including the extra colon)
                                            ui.text_colored(colours.value.value_label, format!("{:ALIGN$}", format!("{}:", $name)));
                                            ui.same_line(); // Keep the metadata value on the same line as the label
                                        }};
                                    }

                                    metadata_label!("source file");
                                    ui.text_colored(colours.value.file_location, metadata.file().unwrap_or("<unknown source file>"));
                                    ui.same_line();
                                    ui.text_colored(colours.text.normal, ":");
                                    ui.same_line();
                                    ui.text_colored(colours.value.file_location, metadata.line().map_or("<unknown line>".to_string(), |line| line.to_string()));
                                    metadata_label!("module path");
                                    ui.text_colored(colours.value.file_location, metadata.module_path().unwrap_or("<unknown module path>"));
                                    metadata_label!("target");
                                    ui.text_colored(colours.value.tracing_event_name, metadata.target());
                                    metadata_label!("level");
                                    ui.text_colored(colours.colour_for_tracing_level(metadata.level()), metadata.level().to_string());
                                    metadata_label!("fields");
                                    let fields = metadata.fields();
                                    if fields.is_empty(){
                                        ui.text_disabled("<None>");
                                    }
                                    else{
                                        // List of names of each field
                                        let names = metadata.fields().iter().map(|f| f.name()).collect_vec();
                                        // Substring of the formatted span fields
                                        let mut fields_substr = formatted_span_fields;
                                        // Loop over in twos; the current field and the next one
                                        for pair in names.windows(2){
                                            let (curr_name, next_name) = (pair[0], pair[1]);
                                            // Check it starts with our current field
                                            if fields_substr.find(curr_name) != Some(0){
                                                // warn!(target: GENERAL_WARNING_NON_FATAL, ?fields_substr, ?curr_name, "substring did not begin with current field");
                                                ui.text_colored(colours.value.tracing_event_field_name, curr_name);
                                                ui.same_line();
                                                ui.text_colored(colours.text.normal, "=");
                                                ui.same_line();
                                                ui.text_disabled("<Missing>");
                                                break;
                                            }
                                            // Remove the current name, and the equals sign (+1).
                                            fields_substr = &fields_substr[(curr_name.len()+1)..];
                                            //Now find where the next field starts in the string
                                            let next_field_match = format!(" {next_name}=");
                                            let next_field_index:usize = match fields_substr.find(&next_field_match){
                                                Some(idx) => idx,
                                                None => {
                                                    // warn!(target: GENERAL_WARNING_NON_FATAL, ?fields_substr, ?curr_name, "could not find index of next field");
                                                    ui.text_colored(colours.value.tracing_event_field_name, curr_name);
                                                    ui.same_line();
                                                    ui.text_colored(colours.text.normal, "=");
                                                    ui.same_line();
                                                    ui.text_disabled("<Parse Error>");
                                                    break;
                                                }
                                            };

                                            // Now we know the current field starts at [0], and we know where the next field starts, so extract that portion of the string
                                            // And we have our current value
                                            // We don't want to include the first char of the next field, so we use a non-inclusive range
                                            let current_value = &fields_substr[0..next_field_index];

                                            // Now substring again so we are ready for the next iteration. +1 removes the spacing between fields
                                            fields_substr = &fields_substr[next_field_index + 1..];

                                            // And now display it
                                            ui.text_colored(colours.value.tracing_event_field_name, curr_name);
                                            ui.same_line();
                                            ui.text_colored(colours.text.normal, "=");
                                            ui.same_line();
                                            ui.text_colored(colours.value.tracing_event_field_value, current_value);
                                            ui.same_line();
                                        }

                                         // Now since [].windows(2) doesn't include the last value, we have to handle that one on it's own
                                         // The string should be the same if there's 1 field (no for loop called) or if there are >1
                                         //field_name=value
                                         // Check it starts with our current field
                                         let last_name = names.last().expect("we should always have at least 1 field name if we got here");
                                         if fields_substr.find(last_name) != Some(0){
                                            ui.text_colored(colours.value.tracing_event_field_name, last_name);
                                            ui.same_line();
                                            ui.text_colored(colours.text.normal, "=");
                                            ui.same_line();
                                            ui.text_disabled("<Missing>");
                                         }
                                         else{
                                            // Remove the current name, and the equals sign (+1).
                                            // This should leave us with just the field value (since it is the only field left in the substring)
                                            fields_substr = &fields_substr[(last_name.len() + 1)..];
                                            let last_value  = fields_substr;

                                             // And now display it
                                             ui.text_colored(colours.value.tracing_event_field_name, last_name);
                                             ui.same_line();
                                             ui.text_colored(colours.text.normal, "=");
                                             ui.same_line();
                                             ui.text_colored(colours.value.tracing_event_field_value, last_value);
                                        }
                                        ui.text(formatted_span_fields);
                                    }
                                    // Omitting metadata.kind() because it's always a span, because we're getting a SpanTrace (duh)
                                    // Same for callsite - doesn't give any useful information (just a pointer to a private struct)
                                    // metadata_label!("callsite");
                                    // ui.text_colored(colours.value.misc_value, format!("{:#?}", metadata.callsite()));
                                    node.end();
                                }
                                depth += 1;
                                span_process_span.exit();
                                true
                            },
                        );
                    }
                }
            } else {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "missing span trace");
                ui.text_colored(
                    colours.severity.neutral,
                    "This error doesn't have a span trace; it was probably captured outside of any spans",
                )
            }
        } else {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "couldn't cast handler");
            ui.text_colored(
                colours.severity.warning,
                "Couldn't downcast error report's handler to get the span trace",
            );
        }
    });
    //TODO: Do we even need this debug section
    section!("Debug", {
        ui.text_colored(colours.value.misc_value, format!("{:#?}", report));
    });
    section!("Stringified", {
        ui.text_colored(
            colours.value.misc_value,
            format_report_string_no_ansi(report),
        );
    });
    //TODO: Report sections
    span_display_error_report.exit();
}
