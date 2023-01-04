//! Module of shared functions used for the UI building
use crate::config::read_config_value;
use crate::config::run_time::keybindings_config::KeyBinding;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_report_string_no_ansi;
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::Report;
use fancy_regex::{Captures, Regex};
use imgui::{Condition, StyleColor, StyleVar, Ui};
use lazy_static::lazy_static;
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
        ($title:literal, $body:expr) => {{
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
        }};
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
                trace!(target: UI_TRACE_BUILD_INTERFACE, "no backtrace: missing");
                ui.text_colored(colours.severity.warning, "This error doesn't have a backtrace. Try checking `RUST_BACKTRACE` and/or `RUST_BACKTRACE` environment variables are set")
            }
        } else {
            trace!(
                target: UI_TRACE_BUILD_INTERFACE,
                "no backtrace: couldn't cast handler"
            );
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
                                let span_process_span = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "process_span", depth, ?metadata, formatted_span_fields=formatted_span_fields.to_owned()).entered();

                                // Construct a tree node with the span name as the title
                                // If the node is expanded, then we get to see all the juicy information
                                let tree_node_colour_style = ui.push_style_color(StyleColor::Text, colours.value.tracing_event_name); //Colour the title
                                let maybe_tree_node = ui.tree_node(format!("{depth}: {name}", name=metadata.name()));
                                tree_node_colour_style.pop();

                                 //ImGUI adds spacing between elements normally, but since I'm trying to pack them together we need to remove that spacing
                                 let no_spacing_style_var = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

                                if let Some(tree_node) = maybe_tree_node {
                                    /*
                                    Here, I want the metadata labels and the values to all be aligned nicely
                                    This is a little tricky to do purely with spaces/tabs, since the fonts might not be monospace
                                    So what we actually do is create a table, which is much nicer
                                    */
                                    if let Some(table_token) = ui.begin_table("span fields table", 2){

                                        ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.value_label, "source file");
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.file_location, metadata.file().unwrap_or("<unknown source file>"));
                                        ui.same_line();
                                        ui.text_colored(colours.text.normal, ":");
                                        ui.same_line();
                                        ui.text_colored(colours.value.file_location, metadata.line().map_or("<unknown line>".to_string(), |line| line.to_string()));

                                            ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.value_label, "module path");
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.file_location, metadata.module_path().unwrap_or("<unknown module path>"));

                                            ui.table_next_column();
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.value_label, "target");
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.tracing_event_name, metadata.target());

                                            ui.table_next_column();
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.value_label, "level");
                                        ui.table_next_column();
                                        ui.text_colored(colours.colour_for_tracing_level(metadata.level()), metadata.level().to_string());

                                            ui.table_next_column();
                                        ui.table_next_column();
                                        ui.text_colored(colours.value.value_label, "fields");
                                        ui.table_next_column();
                                        let fields = metadata.fields();
                                        if fields.is_empty(){
                                            ui.text_disabled("<None>");
                                            }
                                        else{
                                            lazy_static!{
                                                // Mostly working but will have to manually split string?: https://regex101.com/r/KCn0Q1/1
                                                static ref VALUE_EXTRACTOR_REGEX: Regex = Regex::new(indoc::indoc! {r#"
                                                    (?P<field>(?#
                                                    Each field is made up of a key, an equals sign, and a value
                                                    The key is always a single word, underscores allowed - has to be valid rust identifier
                                                    The value can be pretty much any value, including spaces and symbols. Assume that it won't include equals sign, or it gets too tricky to compute
                                                    )(?P<key>(r#)?\w+)=(?P<value>[^=]*?))(?#
                                                    Now we do a positive lookahead to separate the next fields from this field
                                                    Each match *MUST* be followed by either another field, or the end of the string.
                                                    This makes it much easier to match
                                                    Here we repeat <field>, since can't use subroutines/expression references in this dialect of regex
                                                    )(?:$|(?: (?=(?:r#)?\w+=[^=]*?)))"#}).expect("Compile-time regex should be correct");
                                            }
                                            for (index, maybe_capture) in VALUE_EXTRACTOR_REGEX.captures_iter(formatted_span_fields).enumerate(){
                                                // Comma separators between each pair, but not before the first
                                                if index != 0{
                                                    ui.same_line();
                                                    ui.text_colored(colours.text.normal, ", ");
                                                    ui.same_line();
                                                }
                                                // Should give us 3 capture groups: (0) overall match, (1) key, (2) value
                                                match maybe_capture {
                                                    Err(err) =>{
                                                        ui.text_colored(colours.severity.warning, format!("regex captures error: {err}"));
                                                    }
                                                    Ok(capture /*should be called `match`*/) =>{
                                                        let key = capture.name("key");
                                                        match key {
                                                            None => ui.text_disabled("<name?>"),
                                                            Some(key) => ui.text_colored(colours.value.tracing_event_field_name, key.as_str()),
                                                        }
                                                        ui.same_line();
                                                        ui.text_colored(colours.text.normal, "=");
                                                        ui.same_line();
                                                        let val = capture.name("value");
                                                        match val{
                                                            None => ui.text_disabled("<value?>"),
                                                            Some(value) => ui.text_colored(colours.value.tracing_event_field_value, value.as_str()),
                                                        }
                                                    }
                                                }
                                            }
                                        } //end `else` (so if we have fields)
                                        // Omitting metadata.kind() because it's always a span, because we're getting a SpanTrace (duh)
                                        // Same for callsite - doesn't give any useful information (just a pointer to a private struct)
                                        // metadata_label!("callsite");
                                        // ui.text_colored(colours.value.misc_value, format!("{:#?}", metadata.callsite()));
                                        table_token.end();
                                        }
                                        tree_node.end();
                                }
                                no_spacing_style_var.pop();
                                depth += 1;
                                span_process_span.exit();
                                true
                            },
                        );
                    }
                }
            } else {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: non-existent");
                ui.text_colored(
                    colours.severity.neutral,
                    "This error doesn't have a span trace; it was probably captured outside of any spans",
                )
            }
        } else {
            trace!(
                target: UI_TRACE_BUILD_INTERFACE,
                "span trace: couldn't cast handler"
            );
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
