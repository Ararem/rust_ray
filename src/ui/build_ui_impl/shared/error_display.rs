use crate::config::read_config_value;
use crate::config::run_time::ui_config::theme::Theme;
use crate::helper;
use crate::helper::logging::event_targets::*;
use crate::ui::build_ui_impl::shared::constants::{MISSING_VALUE_TEXT, NO_VALUE_TEXT, UNKNOWN_VALUE_TEXT};
use crate::ui::build_ui_impl::shared::{display_c_const_pointer, display_c_mut_pointer, display_maybe_c_mut_pointer, tree_utils};
use backtrace::{BacktraceFrame, BacktraceSymbol};
use color_eyre::section::Section;
use color_eyre::section::SectionExt;
use color_eyre::Report;
use fancy_regex::*;
use helper::logging::*;
use imgui::{Condition, TableFlags, TreeNodeId, Ui};
use indoc::indoc;
use itertools::Itertools;
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Mutex;
use tracing::field::Empty;
use tracing::{trace, trace_span, warn, Metadata, debug};
use tracing_error::SpanTraceStatus;

lazy_static! {
    /// Vector of errors we are currently displaying
    static ref ERRORS: Mutex<Vec<Report>> = Mutex::new(Vec::default());
}
/// Atomic (because it's static) boolean
static SHOW_ERRORS_POPUP: AtomicBool = AtomicBool::new(false);

/// Call this function whenever an error occurs (only call once) and you want to display the error
pub fn an_error_occurred(report: Report) {
    debug!(target: GENERAL_WARNING_NON_FATAL, "received error to display in ui: {report:#}");
    let mut errors_vec = match ERRORS.lock() {
        Ok(lock) => lock,
        Err(err) => {
            warn!(target: GENERAL_WARNING_NON_FATAL, "errors Vec mutex was poisoned by some other thread");
            err.into_inner()
        }
    };
    errors_vec.push(report);
    SHOW_ERRORS_POPUP.store(true, Relaxed);
}

pub fn render_errors_popup(ui: &Ui) {
    const MODAL_NAME: &str = "Error(s)";

    // Open the popup if we need to
    // This is because ImGui owns the popups, not us
    if SHOW_ERRORS_POPUP.swap(false, Relaxed) {
        trace!(target: UI_TRACE_BUILD_INTERFACE, "opening errors popup");
        ui.open_popup(MODAL_NAME);
    }

    trace_span!(target: UI_TRACE_BUILD_INTERFACE, "error_modal").in_scope(|| {
        // If we have an error, its modal time....
        // Also demonstrate passing a bool, this will create a regular close button which
        // will close the popup. Note that the visibility state of popups is owned by imgui, so the input value
        // of the bool actually doesn't matter here.
        let mut opened_sesame = true;
        let popup_token = match ui.modal_popup_config(MODAL_NAME).opened(&mut opened_sesame).begin_popup() {
            None => {
                trace!(target: UI_TRACE_BUILD_INTERFACE, "errors modal not visible");
                return;
            }
            Some(token) => token,
        };
        let mut errors_vec = match ERRORS.lock() {
            Ok(lock) => lock,
            Err(err) => {
                warn!(target: GENERAL_WARNING_NON_FATAL, "errors Vec mutex was poisoned by some other thread");
                err.into_inner()
            }
        };
        let colours = read_config_value(|config| config.runtime.ui.colours);

        if errors_vec.is_empty() {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "errors modal: visible but empty");
            ui.text_colored(colours.text.normal, "No errors to display!\nYou can safely close this window");
            // Here's a little egg for easter I put in here
            let random_chars = (0..=thread_rng().gen_range(12usize..=20usize)) //Generates a random range of 12 to 20 elements
                .map(|_| thread_rng().gen_range('\u{0021}'..='\u{00FF}'))
                .join(""); //Maps each element to a random char in a reasonable range of unicode chars
            ui.text_colored(
                [0.5, 0.5, 0.5, 0.02 /*Almost invisible*/],
                format!(
                    indoc! {r"
                The errors, where have they gone!?

                Perhaps there are none? No, that's not possible. There are always errors somewhere.

                They must be...hiding...yes that's it. They are biding their time, waiting for the perfect opportunity to strike.

                <<Debugging complete>>

                Ooh, I think I've found them! They're in the {}
            "},
                    random_chars
                ),
            );
            popup_token.end();
            return;
        }

        if let Some(tab_bar_token) = ui.tab_bar("Error tab bar") {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "error tab bar visible");
            errors_vec.retain(|report| {
                let span_error_tabs = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "error_tabs", report = format_report_display(report), opened = Empty).entered();
                // This bool is passed into [imgui] when creating each tab, so [imgui] will set it to [false] when the user closes the tab
                // Since we're inside [retain_mut()], we can use this to decide which reports to keep, since it'll only be false once the user closes it
                let mut opened = true;
                let title = format!(
                    "{}",
                    report.chain().next().expect("Every error should have at least one error in the chain, but `.next()` returned [None]")
                );
                if let Some(tab) = ui.tab_item_with_opened(&title, &mut opened) {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "error tab {title} selected");
                    display_eyre_report(ui, report);
                    tab.end();
                } else {
                    trace!(target: UI_TRACE_BUILD_INTERFACE, "error tab {title} not selected");
                }

                // Print the short version of the error to the log, no need for the full one since we had that earlier
                if !opened {
                    trace!(target: UI_DEBUG_USER_INTERACTION, "User hiding error tab {title}");
                }
                span_error_tabs.record("opened", opened);
                span_error_tabs.exit();
                opened
            });
            tab_bar_token.end();
        } else {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "error tab bar hidden");
        }
        popup_token.end();
    });
}

/// Function that displays an [eyre::Report] in [imgui]
///
/// This doesn't create any windows or popups, just renders the error information.
pub fn display_eyre_report(ui: &Ui, report: &Report) {
    /*
    A note on how I've structured this:
    A lot of the UI code requires lots of `unwrap()`s or lots of `if/else`s, which means it gets quite heavily nested
    Most of the time one of the paths is the 'exit' path - it is the exit case where we have a reason not to display anything
    So by putting the code into functions, we can use `return` and massively reduce nesting, since we don't need to nest the proper UI code inside an `if` (sometimes multiple times)
    Before refactoring I went up to 13 indents (13 tabs = 52 spaces)!
    Afterwards, max was 5 tabs
    */

    let span_display_error_report = trace_span!(target: UI_TRACE_BUILD_INTERFACE, "display_error_report").entered();
    let colours = read_config_value(|config| config.runtime.ui.colours);
    macro_rules! section {
        ($title:literal, $body:expr) => {{
            let span_section = trace_span!(target: UI_TRACE_BUILD_INTERFACE, $title).entered();
            let maybe_node = ui.tree_node_config($title).opened(true, Condition::FirstUseEver).push(); // Should be open by default
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

    section!("Backtrace", display_backtrace(ui, &colours, report));
    section!("Span trace", display_span_trace(ui, &colours, report));
    //TODO: Report sections
    span_display_error_report.exit();
}

// ===== BACK TRACE =====
// TODO: Add some tooltips that explain the subtleties and meanings of the backtrace
//  For example, why compressed frames have "outer" prefixing the IP, module addr, and symbol addr,
//  What compressed frames are
//  What unresolved/empty frames are
//  What each of the symbols etc means
fn display_backtrace(ui: &Ui, colours: &Theme, report: &Report) {
    let handler = match report.handler().downcast_ref::<color_eyre::Handler>() {
        // Couldn't downcast to get the handler
        None => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "backtrace: couldn't cast handler");
            ui.text_colored(colours.severity.warning, "Couldn't downcast error report's handler to get the backtrace");
            return;
        }
        Some(handler) => handler,
    };

    let backtrace = match handler.backtrace() {
        None => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "backtrace: non-existent");
            ui.text_colored(
                colours.severity.warning,
                "This error doesn't have a backtrace. Try checking `RUST_BACKTRACE` and/or `RUST_BACKTRACE` environment variables are set",
            );
            return;
        }
        Some(backtrace) => backtrace,
    };

    for (index, frame) in backtrace.frames().iter().enumerate() {
        /*
        We have a minor problem with displaying the backtrace frames: each frame doesn't *always* actually correspond to a single function
        From the docs ([backtrace::BacktraceFrame::symbols()], https://docs.rs/backtrace/latest/backtrace/struct.BacktraceFrame.html#method.symbols):
        > Normally there is only one symbol per frame, but sometimes if a number of functions are inlined into one frame
        > then multiple symbols will be returned. The first symbol listed is the “innermost function”, whereas the last symbol is the outermost (last caller).
        > Note that if this frame came from an unresolved backtrace then this will return an empty list.
        So there's a chance that we'll have multiple symbols (aka function calls) compressed into a single stack frame

        In order to solve this, I've decided to split these compressed frames into sub-frames, i.e. Frame 51.0, 51.1, 51.2 etc
        This means that normal singular frames should be fine
         */
        match frame.symbols().len() {
            0 => display_empty_frame(ui, colours, index, frame),
            1 => display_single_frame(ui, colours, index, frame),
            _ => display_compressed_frame(ui, colours, index, frame),
        }
    }

    /// Displays an empty [BacktraceFrame] (one that has no symbols associated with it)
    /// This should only happen:
    /// > If this frame came from an unresolved backtrace
    fn display_empty_frame(ui: &Ui, colours: &Theme, index: usize, frame: &BacktraceFrame) {
        let instruction_pointer: *mut c_void = frame.ip();
        let symbol_address: *mut c_void = frame.symbol_address();
        let module_base_address: Option<*mut c_void> = frame.module_base_address();
        //[frame.symbols()] is empty

        let maybe_tree_node = tree_utils::tree_node_with_custom_text(
            ui,
            TreeNodeId::<&str>::Ptr(frame as *const BacktraceFrame as *const c_void), // Use the BacktraceFrame for the node's ID
        );

        ui.text_colored(colours.value.value_label, "Frame");
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.number, format!("{index:>2}")); // The 'depth' of the stack frane
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, ":\t"); // The 'depth' of the stack frane
        ui.same_line_with_spacing(0.0, 0.0);
        display_c_mut_pointer(ui, colours, instruction_pointer);
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, " - ");
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.severity.warning, "<Unresolved>");

        let tree_node = match maybe_tree_node {
            None => return,
            Some(node) => node,
        };

        /*
        Here, I want the metadata labels and the values to all be aligned nicely
        This is a little tricky to do purely with spaces/tabs, since the fonts might not be monospace
        So what we actually do is create a table, which is much nicer, and prettier
        The TableFlags make the table waste much less space on the metadata label columns
        */
        let table_token = match ui.begin_table_with_flags("span fields table", 2, TableFlags::SIZING_FIXED_FIT) {
            None => {
                // I'm not sure why this would be [None], but just in case
                return;
            }
            Some(token) => token,
        };

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "instruction pointer");
        ui.table_next_column();
        display_c_mut_pointer(ui, colours, instruction_pointer);

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "symbol address");
        ui.table_next_column();
        display_c_mut_pointer(ui, colours, symbol_address);

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "module base address");
        ui.table_next_column();
        display_maybe_c_mut_pointer(ui, colours, module_base_address);

        table_token.end();
        tree_node.end();
    }

    fn display_single_frame(ui: &Ui, colours: &Theme, index: usize, frame: &BacktraceFrame) {
        let frame_instruction_pointer: *mut c_void = frame.ip();
        let frame_symbol_address: *mut c_void = frame.symbol_address();
        let frame_module_base_address: Option<*mut c_void> = frame.module_base_address();
        let frame_index_str = format!("{index:>2}");

        display_symbol_frame(
            ui,
            colours,
            &frame_index_str,
            &frame.symbols()[0],
            frame_instruction_pointer,
            frame_symbol_address,
            frame_module_base_address,
        );
    }

    /// Displays a 'compressed' [BacktraceFrame] (one that has more than one symbols associated with it)
    ///
    /// From the [backtrace] docs:
    /// > Normally there is only one symbol per frame, but sometimes if a number
    /// > of functions are inlined into one frame then multiple symbols will be
    /// > returned. The first symbol listed is the "innermost function", whereas
    /// > the last symbol is the outermost (last caller).
    fn display_compressed_frame(ui: &Ui, colours: &Theme, frame_index: usize, frame: &BacktraceFrame) {
        let frame_instruction_pointer: *mut c_void = frame.ip();
        let frame_symbol_address: *mut c_void = frame.symbol_address();
        let frame_module_base_address: Option<*mut c_void> = frame.module_base_address();

        for (sub_frame_index, symbol) in frame.symbols().iter().enumerate() {
            let frame_index_str = format!("{frame_index:>2}.{sub_frame_index}");
            display_symbol_frame(ui, colours, &frame_index_str, symbol, frame_instruction_pointer, frame_symbol_address, frame_module_base_address);
        }
    }

    /// The shared function called by
    fn display_symbol_frame(
        ui: &Ui,
        colours: &Theme,
        frame_index_str: &str,
        symbol: &BacktraceSymbol,
        frame_instruction_pointer: *mut c_void,
        frame_symbol_address: *mut c_void,
        frame_module_base_address: Option<*mut c_void>,
    ) {
        let maybe_tree_node = tree_utils::tree_node_with_custom_text(
            ui,
            TreeNodeId::<&str>::Ptr(symbol as *const BacktraceSymbol as *const c_void), // Use the BacktraceSymbol for the node's ID
        );

        ui.text_colored(colours.value.value_label, "Frame ");
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.number, frame_index_str);
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, ":\t");
        ui.same_line_with_spacing(0.0, 0.0);
        display_c_mut_pointer(ui, colours, frame_instruction_pointer);
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, " - ");
        ui.same_line_with_spacing(0.0, 0.0);
        // Demangled symbol name in title
        if let Some(ref symbol_name) = symbol.name() {
            let full = symbol_name.to_string(); // Use to_string() for demangled name
            let mut short = String::with_capacity(128);
            let mut generic_depth: u64 = 0; // u8 since we should never go >256 and it should always be +ve, -ve is an error
                                            // Here we process the symbol name to make it easier to read
                                            // Remove the generic type arguments
                                            // And any `::impl$XXX::` parts
            let mut index: usize = 0;
            'shorten: loop {
                let segment = &full[index..];
                if index >= full.len() {
                    // Got to the end of the string
                    break 'shorten;
                }

                let char = match &segment.chars().next() {
                    None => break warn!(target: GENERAL_WARNING_NON_FATAL, full, short, segment, index, "didn't have a char at index {index}"),
                    Some(c) => *c,
                };

                if char == '<' {
                    generic_depth += 1;
                    index += 1;
                    continue;
                }
                if char == '>' {
                    generic_depth -= 1;
                    index += 1;
                    continue;
                }

                if generic_depth != 0 {
                    index += 1;
                    continue;
                }

                if segment.starts_with("::impl$") {
                    // Find where the next part of the path starts
                    // By skipping to the next colon
                    index += 7; // Skip the chars of `::impl$`
                    let next_colon_index = match full[index..].find(':') {
                        None => {
                            break warn!(
                                target: GENERAL_WARNING_NON_FATAL,
                                full, short, segment, index, "didn't find next colon - symbol path seems to end with impl block. this shouldn't happen"
                            )
                        }
                        Some(idx) => index + idx,
                    };
                    index = next_colon_index;
                    continue 'shorten;
                }

                //If we get here, we're skipped all the unnecessary chars, so add this ones
                short.push(char);
                index += 1;
            }

            ui.text_colored(colours.value.function_name, short);
        } else {
            ui.text_colored(colours.severity.warning, UNKNOWN_VALUE_TEXT);
        }

        // Only continue if node is opened
        let tree_node = match maybe_tree_node {
            None => return,
            Some(node) => node,
        };
        /*
        Here, I want the metadata labels and the values to all be aligned nicely
        This is a little tricky to do purely with spaces/tabs, since the fonts might not be monospace
        So what we actually do is create a table, which is much nicer, and prettier
        The TableFlags make the table waste much less space on the metadata label columns
        */
        let table_token = match ui.begin_table_with_flags("span fields table", 2, TableFlags::SIZING_FIXED_FIT) {
            None => {
                // I'm not sure why this would be [None], but just in case
                return;
            }
            Some(token) => token,
        };

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "file location");
        ui.table_next_column();
        if let Some(filename) = symbol.filename() {
            ui.text_colored(colours.value.file_location, filename.display().to_string());
        } else {
            ui.text_colored(colours.value.missing_value, "<Unknown file>");
        }
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, ":");
        ui.same_line_with_spacing(0.0, 0.0);
        if let Some(line) = symbol.lineno() {
            ui.text_colored(colours.value.file_location, format!("l{line}"));
        } else {
            ui.text_colored(colours.value.missing_value, "???");
        }
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, ":");
        ui.same_line_with_spacing(0.0, 0.0);
        if let Some(column) = symbol.colno() {
            ui.text_colored(colours.value.file_location, format!("c{column}"));
        } else {
            ui.text_colored(colours.value.missing_value, "???");
        }

        if let Some(ref symbol_name) = symbol.name() {
            let demangled = format!("{}", symbol_name /*Display trait gives demangled name*/);
            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "symbol name (demangled)");
            ui.table_next_column();
            ui.text_colored(colours.value.function_name, &demangled);

            // Although [as_str()] should return the mangled name according to the docs, it doesn't seem to on windows (https://github.com/rust-lang/backtrace-rs/issues/36#issuecomment-285390548):
            // > Because I tested my code on MacOS and there the "as_str()" function returned the demangled names,
            // but on Linux that did not work :( Now, I am using the "to_string()" function, but to find this bug took a lot of time :D
            // So if the mangled and demangled names differ, print the mangled one here
            // This helps avoid duplicate names in the UI
            if let Some(mangled) = symbol_name.as_str() {
                if mangled != demangled {
                    ui.table_next_row();
                    ui.table_next_column();
                    ui.text_colored(colours.value.value_label, "symbol name (mangled)");
                    ui.table_next_column();
                    ui.text_colored(colours.value.function_name, mangled);
                }
            }
        }
        //[symbol.name()] returned [None]
        else {
            ui.table_next_row();
            ui.table_next_column();
            ui.text_colored(colours.value.value_label, "symbol name");
            ui.table_next_column();
            ui.text_colored(colours.severity.warning, UNKNOWN_VALUE_TEXT);
        }

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "outer instruction pointer");
        ui.table_next_column();
        display_c_mut_pointer(ui, colours, frame_instruction_pointer);

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "outer symbol address");
        ui.table_next_column();
        display_c_const_pointer(ui, colours, frame_symbol_address);

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "outer module base address");
        ui.table_next_column();
        display_maybe_c_mut_pointer(ui, colours, frame_module_base_address);

        ui.table_next_row();
        ui.table_next_column();
        ui.text_colored(colours.value.value_label, "symbol address");
        ui.table_next_column();
        display_maybe_c_mut_pointer(ui, colours, symbol.addr());

        table_token.end();
        tree_node.end();
    }
}

// ===== SPAN TRACE =====
fn display_span_trace(ui: &Ui, colours: &Theme, report: &Report) {
    let handler = match report.handler().downcast_ref::<color_eyre::Handler>() {
        // Couldn't downcast to get the handler
        None => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: couldn't cast handler");
            ui.text_colored(colours.severity.warning, "Couldn't downcast error report's handler to get the span trace");
            return;
        }
        Some(handler) => handler,
    };

    let span_trace = match handler.span_trace() {
        None => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: non-existent");
            ui.text_colored(colours.value.missing_value, "This error doesn't have a span trace; it was probably captured outside of any spans");
            return;
        }
        Some(span_trace) => span_trace,
    };

    match span_trace.status() {
        SpanTraceStatus::UNSUPPORTED => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: not supported");
            ui.text_colored(
                colours.severity.warning,
                "SpanTraces are not supported, likely because there is no [ErrorLayer] or the [ErrorLayer] is from a different version of [tracing_error]",
            );
            return;
        }
        SpanTraceStatus::EMPTY => {
            trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: empty");
            ui.text_colored(colours.severity.warning, "The SpanTrace is empty, likely because it was captured outside of any spans");
            return;
        }
        _ => (),
    };
    trace!(target: UI_TRACE_BUILD_INTERFACE, "span trace: captured");
    // [with_spans] calls the closure on every span in the trace, as long as the closure returns `true`
    let mut depth = 0;
    span_trace.with_spans(|metadata: &'static Metadata<'static>, formatted_span_fields: &str| -> bool {
        visit_each_span(ui, colours, metadata, formatted_span_fields, depth);
        depth += 1;
        true
    });
}

/// 'Visits' each span in the span-trace, and displays it in the ui
fn visit_each_span(ui: &Ui, colours: &Theme, metadata: &'static Metadata<'static>, formatted_span_fields: &str, depth: i32) {
    let span_visit_span = trace_span!(
        target: UI_TRACE_BUILD_INTERFACE,
        "visit_span",
        depth,
        ?metadata,
        formatted_span_fields = formatted_span_fields.to_owned()
    )
    .entered();
    let maybe_tree_node = tree_utils::tree_node_with_custom_text(ui, metadata.name());

    // Fancy colours are always better than simple ones right?
    ui.text_colored(colours.value.value_label, "Span ");
    ui.same_line_with_spacing(0.0, 0.0);
    ui.text_colored(colours.value.number, format!("{depth:>2}"));
    ui.same_line_with_spacing(0.0, 0.0);
    ui.text_colored(colours.value.symbol, ":\t");
    ui.same_line_with_spacing(0.0, 0.0);
    ui.text_colored(colours.value.tracing_event_name, metadata.name());

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
    let table_token = match ui.begin_table_with_flags("span fields table", 2, TableFlags::SIZING_FIXED_FIT) {
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
    ui.text_colored(colours.value.file_location, metadata.file().unwrap_or("<unknown source file>"));
    ui.same_line_with_spacing(0.0, 0.0);
    ui.text_colored(colours.value.symbol, ":");
    ui.same_line_with_spacing(0.0, 0.0);
    ui.text_colored(colours.value.file_location, metadata.line().map_or("<unknown line>".to_string(), |line| line.to_string()));

    ui.table_next_row();
    ui.table_next_column();
    ui.text_colored(colours.value.value_label, "module path");
    ui.table_next_column();
    ui.text_colored(colours.value.file_location, metadata.module_path().unwrap_or("<unknown module path>"));

    ui.table_next_row();
    ui.table_next_column();
    ui.text_colored(colours.value.value_label, "target");
    ui.table_next_column();
    ui.text_colored(colours.value.tracing_event_name, metadata.target());

    ui.table_next_row();
    ui.table_next_column();
    ui.text_colored(colours.value.value_label, "level");
    ui.table_next_column();
    ui.text_colored(colours.colour_for_tracing_level(metadata.level()), metadata.level().to_string());

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

    span_visit_span.exit();
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

/// Takes in the formatted representation of the span fields, and parses it into a map of field names and field values (may be multiple values per name)
fn parse_span_fields<'field>(formatted_span_fields: &'field str) -> HashMap<&'field str, Vec<&'field str>> {
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
                        warn!(
                            target: GENERAL_WARNING_NON_FATAL,
                            "cannot have a match for <field> without also having a match for <key>: `capture.name(\"key\")` returned [None]"
                        );
                        continue;
                    }
                    Some(key) => key.as_str(),
                };
                let value = match capture.name("value") {
                    None => {
                        warn!(
                            target: GENERAL_WARNING_NON_FATAL,
                            "cannot have a match for <field> without also having a match for <value>: `capture.name(\"value\")` returned [None]"
                        );
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

fn display_span_fields<'field>(ui: &Ui, colours: &Theme, fields: Vec<ProcessedSpanField<'field>>) {
    if fields.is_empty() {
        // Only should be empty if there should be, and are no fields
        ui.text_colored(colours.value.missing_value, NO_VALUE_TEXT);
        if ui.is_item_hovered() {
            ui.tooltip_text("This span doesn't have any fields");
        }
        return;
    }
    for field in fields.iter() {
        // Removed this because it broke when we had multiple values per field
        // Also not really necessary
        // // Display comma separators between each pair, but not before the first
        // if field_index != 0 {
        //     ui.same_line_with_spacing(0.0, 0.0);
        //     ui.text_colored(colours.value.symbol, ", ");
        // }
        if field.valid {
            ui.text_colored(colours.value.tracing_event_field_name, field.name);
        } else {
            ui.text_colored(colours.severity.warning, field.name);
            if ui.is_item_hovered() {
                ui.tooltip_text("This field doesn't exist in the original span metadata. There was likely an error parsing the span's fields, and some of the fields may be incorrect");
            }
        }
        ui.same_line_with_spacing(0.0, 0.0);
        ui.text_colored(colours.value.symbol, "=");
        ui.same_line_with_spacing(0.0, 0.0);
        match &field.values {
            SpanFieldValue::Missing => {
                ui.text_colored(colours.severity.warning, MISSING_VALUE_TEXT);
                if ui.is_item_hovered() {
                    // TODO: This seems to be a bug with the [ErrorLayer]
                    //  I've done testing by explicitly recording a field before the error occurs and it's still marked as empty
                    //  My assumption is that [ErrorLayer] is a bit "dumb" and only records the fields when the span enters, and never changes them again
                    //  So it does nothing when `.record("field", value)` is called
                    // TODO: Either create an issue report with them, or (preferably) implement a custom [ErrorLayer]/[Formatter] that's not completely terrible
                    //  Because their default one really is atrocious
                    ui.tooltip_text("This field exists in the span's metadata, but was [Empty] because it wasn't assigned on span creation. This is a bug from [tracing_error]");
                }
            }
            SpanFieldValue::Single(val) => {
                ui.text_colored(colours.value.tracing_event_field_value, val);
            }
            SpanFieldValue::Multiple(values) => {
                let group = ui.begin_group();
                for (val_index, &val) in values.iter().enumerate() {
                    ui.text_colored(colours.value.tracing_event_field_value, val);
                    // Put commas at the end of each value, except the last
                    if val_index < values.len() - 1 {
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.text_colored(colours.value.symbol, ",");
                    }
                }
                group.end();
                if ui.is_item_hovered() {
                    ui.tooltip_text("This field has multiple values. Each value is listed on it's own line");
                }
            }
        }
    }
}

fn process_span_fields<'field>(metadata: &'static Metadata<'static>, mut fields_map: HashMap<&'field str, Vec<&'field str>>) -> Vec<ProcessedSpanField<'field>> {
    let mut fields: Vec<ProcessedSpanField<'field>> = vec![];
    // Loop over each field that we *should* have, according to the metadata
    for meta_field in metadata.fields() {
        let name = meta_field.name();
        // Try and extract the entry from the fields map that corresponds to the field in the metadata
        // If the entry is [None], it means that we didn't parse a field with that name
        // Which means that the field wasn't recorded
        let field_value = match fields_map.remove(name) {
            None => SpanFieldValue::Missing,
            Some(values) if values.is_empty() => SpanFieldValue::Missing,
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
        warn!(
            target: GENERAL_WARNING_NON_FATAL,
            "had leftover fields that were parsed but not present in metadata. likely this means the source string was not parsed correctly"
        );
    }
    for (name, values) in fields_map {
        let values = match values.len() {
            0 => SpanFieldValue::Missing,
            1 => SpanFieldValue::Single(values[0]),
            _ => SpanFieldValue::Multiple(values),
        };
        fields.push(ProcessedSpanField::<'field> { name, values, valid: false });
    }

    fields
}
