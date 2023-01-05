//! Module of shared functions used for the UI building
use crate::config::read_config_value;
use crate::config::run_time::keybindings_config::KeyBinding;
use crate::config::run_time::ui_config::theme::*;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::{format_report_display, format_report_string_no_ansi};
use crate::ui::build_ui_impl::UiItem;
use crate::FallibleFn;
use color_eyre::{section::IndentedSection, Report, Section, SectionExt};
use fancy_regex::Regex;
use imgui::{Condition, StyleColor, StyleVar, TableFlags, Ui};
use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::{debug, trace, trace_span, warn, Metadata};
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
    /*
    A note on how I've structured this:
    A lot of the UI code requires lots of `unwrap()`s or lots of `if/else`s, which means it gets quite heavily nested
    Most of the time one of the paths is the 'exit' path - it is the exit case where we have a reason not to display anything
    So by putting the code into functions, we can use `return` and massively reduce nesting, since we don't need to nest the proper UI code inside an `if` (sometimes multiple times)
    Before refactoring I went up to 13 indents (13 tabs = 52 spaces)!
    Afterwards, max was 5 tabs
    */

    let span_display_error_report =
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "display_error_report").entered();
    let colours = read_config_value(|config| config.runtime.ui.colours);
    macro_rules! section {
        ($title:literal, $body:expr) => {{
            let span_section = trace_span!(target: UI_TRACE_BUILD_INTERFACE, $title).entered();
            let maybe_node = ui
                .tree_node_config($title)
                .opened(true, Condition::FirstUseEver)
                .push(); // Should be open by default
            if let Some(opened_node) = maybe_node {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "node expanded");
                $body;
                opened_node.end();
            } else {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "node closed");
            }

            span_section.exit();
        }};
    }
    section!("Doesnt work - closure", {
        let x = ||{
                      format!("");
            format!("");
                                    fn why_is_this_not() {}
        }
    });
    // Works
    let x = || {
        format!("");
        format!("");
        fn why_is_this_not() {}
    };
    // Works
    {
        format!("");
        format!("");
        fn why_is_this_not() {}
    };
    section!("Works - no closure", {
        {
            format!("");
            format!("");
            fn why_is_this_not() {}
        }
    });
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
    section!("Span trace", display_span_trace(ui, &colours, report));
    fn display_span_trace(ui: &Ui, colours: &Theme, report: &Report) {
        let handler = match report.handler().downcast_ref::<color_eyre::Handler>() {
            // Couldn't downcast to get the handler
            None => {
                trace!(
                    target: UI_TRACE_BUILD_INTERFACE,
                    "span trace: couldn't cast handler"
                );
                ui.text_colored(
                    colours.severity.warning,
                    "Couldn't downcast error report's handler to get the span trace",
                );
                return;
            }
            Some(handler) => handler,
        };

        let span_trace = match handler.span_trace() {
            None => {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: non-existent");
                ui.text_colored(
                    colours.severity.neutral,
                    "This error doesn't have a span trace; it was probably captured outside of any spans",
                );
                return;
            }
            Some(span_trace) => span_trace,
        };

        match span_trace.status() {
            SpanTraceStatus::UNSUPPORTED => {
                trace!(
                    target: UI_TRACE_BUILD_INTERFACE,
                    "span trace: not supported"
                );
                ui.text_colored(colours.severity.warning, "SpanTraces are not supported, likely because there is no [ErrorLayer] or the [ErrorLayer] is from a different version of [tracing_error]");
                return;
            }
            SpanTraceStatus::EMPTY => {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: empty");
                ui.text_colored(
                    colours.severity.warning,
                    "The SpanTrace is empty, likely because it was captured outside of any spans",
                );
                return;
            }
            _ => (),
        };
        trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: captured");
        // [with_spans] calls the closure on every span in the trace, as long as the closure returns `true`
        let mut depth = 0;
        span_trace.with_spans(
            |metadata: &'static Metadata<'static>, formatted_span_fields: &str| -> bool {
                visit_each_span(ui, colours, metadata, formatted_span_fields, depth);
                depth += 1;
                true
            },
        );

        /// 'Visits' each span in the span-trace, and displays it in the ui
        fn visit_each_span(
            ui: &Ui,
            colours: &Theme,
            metadata: &'static Metadata<'static>,
            formatted_span_fields: &str,
            depth: i32,
        ) {
            let span_visit_span = trace_span!(
                target: UI_TRACE_BUILD_INTERFACE,
                "visit_span",
                depth,
                ?metadata,
                formatted_span_fields = formatted_span_fields.to_owned()
            )
            .entered();

            /// Takes in the formatted representation of the span fields, and parses it into a map of field names and field values (may be multiple values per name)
            fn parse_span_fields<'field>(
                formatted_span_fields: &'field str,
            ) -> HashMap<&'field str, Vec<&'field str>> {
                // The [HashMap] we store our fields in
                // We use a [Vec<String>] for the value because although not explicitly stated, the default [eyre] formatter just continually appends to it's internal String buffer
                // This means that every time we `.record()` a field, it just adds on that value to the string, and doesn't remove the old one
                // So, we can get multiple fields with the same name but different values here
                // So just in case, we have to account for that and use a Vec
                let mut field_map: HashMap<&'field str, Vec<&'field str>> = HashMap::new();

                // Now we match our [Regex] (technically our [fancy_regex::Regex]) to the string, and extract the named captures
                for maybe_capture in VALUE_EXTRACTOR_REGEX.captures_iter(formatted_span_fields) {
                    // Should give us 3 capture groups: (0) overall match, (1) key, (2) value
                    match maybe_capture {
                        Err(err) => {
                            warn!(
                                target: GENERAL_WARNING_NON_FATAL,
                                report = format_report_display(
                                    &Report::new(err)
                                    .wrap_err("encountered error when matching value extractor regex to formatted fields string")
                                    .section(VALUE_EXTRACTOR_REGEX.as_str().header("value extractor regex:"))
                                    .section(formatted_span_fields.to_owned().header("input string:"))
                                )
                            );
                        }
                        Ok(capture /*should be called `match`*/) => {
                            let key = match capture.name("key") {
                                None => {
                                    warn!(target: GENERAL_WARNING_NON_FATAL, "cannot have a match for <field> without also having a match for <key>: `capture.name(\"key\")` returned [None]");
                                    continue;
                                }
                                Some(key) => key.as_str(),
                            };
                            let value = match capture.name("value") {
                                None => {
                                    warn!(target: GENERAL_WARNING_NON_FATAL, "cannot have a match for <field> without also having a match for <value>: `capture.name(\"value\")` returned [None]");
                                    continue;
                                }
                                Some(value) => value.as_str(),
                            };
                            field_map.entry(key).or_default().push(value);
                        }
                    }
                }

                return field_map;

                lazy_static! {
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
            }

            struct ProcessedSpanField<'field> {
                /// The name of the field that has been processed
                name: &'field str,
                values: SpanFieldValue<'field>,
                valid: bool,
            }
            enum SpanFieldValue<'field> {
                /// The field was assigned [tracing::field::Empty], and wasn't recorded yet
                Missing,
                /// A single field was recorded, standard behaviour
                Single(&'field str),
                /// Multiple values were recorded for this field.
                Multiple(Vec<&'field str>),
            }
            fn process_span_fields<'field>(
                metadata: &'static Metadata<'static>,
                mut fields_map: HashMap<&'field str, Vec<&'field str>>,
            ) -> Vec<ProcessedSpanField<'field>> {
                let mut fields: Vec<ProcessedSpanField<'field>> = vec![];
                // Loop over each field that we *should* have, according to the metadata
                for meta_field in metadata.fields() {
                    let name = meta_field.name();
                    // Try and extract the entry from the fields map that corresponds to the field in the metadata
                    // If the entry is [None], it means that we didn't parse a field with that name
                    // Which means that the field wasn't recorded
                    let field_value = match fields_map.remove(name) {
                        None => SpanFieldValue::Missing,
                        Some(values) if values.len() == 0 => SpanFieldValue::Missing,
                        Some(values) if values.len() == 1 => SpanFieldValue::Single(values[0]),
                        Some(values) => SpanFieldValue::Multiple(values),
                    };
                    fields.push(ProcessedSpanField::<'field> {
                        name,
                        values: field_value,
                        valid: true,
                    });
                }
                // Now we go through and check any remaining fields that exist in the hashmap
                // There shouldn't be any, since I'm not aware of any way that fields can be added to the string without also being present in the metadata
                // I believe this may occur however if the string is incorrectly parsed
                if !fields_map.is_empty() {
                    warn!(target: GENERAL_WARNING_NON_FATAL, "had leftover fields that were parsed but not present in metadata. likely this means the source string was not parsed correctly");
                }
                for (name, values) in fields_map {
                    let values = match values.len() {
                        0 => SpanFieldValue::Missing,
                        1 => SpanFieldValue::Single(values[0]),
                        _ => SpanFieldValue::Multiple(values),
                    };
                    fields.push(ProcessedSpanField::<'field> {
                        name,
                        values,
                        valid: false,
                    });
                }

                fields
            }

            fn display_span_fields<'field>(
                ui: &Ui,
                colours: &Theme,
                fields: Vec<ProcessedSpanField<'field>>,
            ) {
                if fields.is_empty() {
                    // Only should be empty if there should be, and are no fields
                    ui.text_colored(colours.value.missing_value, "<None>");
                    if ui.is_item_hovered() {
                        ui.tooltip_text("This span doesn't have any fields");
                    }
                    return;
                }
                for (field_index, field) in fields.iter().enumerate() {
                    // Display comma separators between each pair, but not before the first
                    // ~~This also keeps every field on the same line~~
                    if field_index != 0 {
                        ui.same_line();
                        ui.text_colored(colours.value.symbol, ", ");
                        // ui.same_line();
                    }
                    if field.valid {
                        ui.text_colored(colours.value.tracing_event_field_name, field.name);
                    } else {
                        ui.text_colored(colours.severity.warning, field.name);
                        if ui.is_item_hovered() {
                            ui.tooltip_text("This field doesn't exist in the original span metadata. There was likely an error parsing the span's fields, and some of the fields may be incorrect");
                        }
                    }
                    ui.same_line();
                    ui.text_colored(colours.value.symbol, "=");
                    ui.same_line();
                    match &field.values {
                        SpanFieldValue::Missing => {
                            ui.text_colored(colours.severity.warning, "<missing>");
                            if ui.is_item_hovered() {
                                ui.tooltip_text("This field exists in the span's metadata, but was empty when the error occurred. It probably wasn't recorded before the error happened.");
                            }
                        }
                        SpanFieldValue::Single(val) => {
                            ui.text_colored(colours.value.tracing_event_field_value, val);
                        }
                        SpanFieldValue::Multiple(values) => {
                            let group = ui.begin_group();
                            ui.text_colored(colours.value.symbol, "[");
                            for &val in values.iter() {
                                ui.text_colored(
                                    colours.value.tracing_event_field_value,
                                    format!("\t{}", val),
                                );
                            }
                            ui.text_colored(colours.value.symbol, "]");
                            group.end();
                            if ui.is_item_hovered() {
                                ui.tooltip_text("This field has multiple values. Each value is listed on it's own line");
                            }
                        }
                    }
                }
            }

            // Construct a tree node with the span name as the title
            // If the node is expanded, then we get to see all the juicy information
            // Note this ordering of the next ~5 lines is important (style var calls in relation to tree node calls)
            let tree_node_colour_style =
                ui.push_style_color(StyleColor::Text, colours.value.tracing_event_name); //Colour the title
            let maybe_tree_node = ui.tree_node(format!("{depth}: {name}", name = metadata.name()));
            tree_node_colour_style.pop();

            //ImGUI adds spacing between elements normally, but since I'm trying to pack them together we need to remove that spacing
            let no_spacing_style_var = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));

            let tree_node = match maybe_tree_node {
                None => {
                    // This specific span's node is closed
                    return;
                }
                Some(node) => node,
            };

            /*
            Here, I want the metadata labels and the values to all be aligned nicely
            This is a little tricky to do purely with spaces/tabs, since the fonts might not be monospace
            So what we actually do is create a table, which is much nicer, and prettier
            The TableFlags make the table waste much less space on the metadata label columns
            */
            let table_token = match ui.begin_table_with_flags(
                "span fields table",
                2,
                TableFlags::SIZING_FIXED_FIT,
            ) {
                None => {
                    // I'm not sure why this would be [None], but just in case
                    return;
                }
                Some(token) => token,
            };
            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "source file");
            ui.table_next_column();
            ui.text_colored(
                colours.value.file_location,
                metadata.file().unwrap_or("<unknown source file>"),
            );
            ui.same_line();
            ui.text_colored(colours.value.symbol, ":");
            ui.same_line();
            ui.text_colored(
                colours.value.file_location,
                metadata
                    .line()
                    .map_or("<unknown line>".to_string(), |line| line.to_string()),
            );

            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "module path");
            ui.table_next_column();
            ui.text_colored(
                colours.value.file_location,
                metadata.module_path().unwrap_or("<unknown module path>"),
            );

            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "target");
            ui.table_next_column();
            ui.text_colored(colours.value.tracing_event_name, metadata.target());

            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "level");
            ui.table_next_column();
            ui.text_colored(
                colours.colour_for_tracing_level(metadata.level()),
                metadata.level().to_string(),
            );

            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "fields");
            ui.table_next_column();
            let fields_map = parse_span_fields(formatted_span_fields);
            let processed_fields = process_span_fields(metadata, fields_map);
            display_span_fields(ui, colours, processed_fields);

            // Omitting metadata.kind() because it's always a span, because we're getting a SpanTrace (duh)
            // Same for callsite - doesn't give any useful information (just a pointer to a private struct)
            // metadata_label!("callsite");
            // ui.text_colored(colours.value.misc_value, format!("{:#?}", metadata.callsite()));

            table_token.end();
            tree_node.end();
            no_spacing_style_var.pop();

            span_visit_span.exit();
        } //end visit_each_span()
    } //end display_span_trace()
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
