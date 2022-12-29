mod config_ui_impl;
mod ui_management;

use crate::config::read_config_value;
use crate::config::run_time::keybindings_config::KeyBinding;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::span_time_elapsed_field::SpanTimeElapsedField;
use crate::program::thread_messages::ProgramThreadMessage::*;
use crate::program::thread_messages::QuitAppNoErrorReason::QuitInteractionByUser;
use crate::program::thread_messages::ThreadMessage::*;
use crate::program::thread_messages::*;
use crate::ui::ui_data::UiData;
use crate::ui::ui_system::UiManagers;
use crate::FallibleFn;
use config_ui_impl::render_config_ui;
use imgui::Condition;
use indoc::*;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use tracing::field::*;
use tracing::*;

pub trait UiItem {
    fn render(&mut self, ui: &imgui::Ui, visible: bool) -> FallibleFn;
}

fn build_window<T: UiItem>(
    label: &str,
    item: &mut T,
    opened: &mut bool,
    ui: &imgui::Ui,
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
fn build_window_fn(
    label: &str,
    func: fn(&imgui::Ui, bool) -> FallibleFn,
    opened: &mut bool,
    ui: &imgui::Ui,
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

pub(super) fn build_ui(
    ui: &imgui::Ui,
    managers: &mut UiManagers,
    data: &mut UiData,
    message_sender: &BroadcastSender<ThreadMessage>,
    message_receiver: &BroadcastReceiver<ThreadMessage>,
) -> FallibleFn {
    //Makes it easier to separate out frames
    trace!(
        target: UI_TRACE_BUILD_INTERFACE,
        "{0} BEGIN BUILD FRAME {frame} {0}",
        str::repeat("=", 50),
        frame = ui.frame_count()
    );
    let timer = SpanTimeElapsedField::new();
    let span_build_ui = trace_span!(
        target: UI_TRACE_BUILD_INTERFACE,
        "build_ui",
        elapsed = Empty
    )
    .entered();

    const NO_SHORTCUT: &str = "N/A"; // String that we use as the shortcut text when there isn't one

    // refs to reduce clutter
    let show_demo_window = &mut data.windows.show_demo_window;
    let show_metrics_window = &mut data.windows.show_metrics_window;
    let show_ui_management_window = &mut data.windows.show_ui_management_window;
    let show_config_window = &mut data.windows.show_config_window;
    let keys = read_config_value(|config| config.runtime.keybindings);

    trace_span!(target: UI_TRACE_BUILD_INTERFACE, "main_menu_bar").in_scope(|| {
        let toggle_menu_item =
            |name: &str, toggle: &mut bool, maybe_shortcut: &Option<&KeyBinding>, tooltip: &str| {
                let span_create_toggle_menu_item = trace_span!(
                    target: UI_TRACE_BUILD_INTERFACE,
                    "create_toggle_menu_item",
                    name,
                    toggle,
                )
                .entered();

                // Using build_with_ref makes a nice little checkmark appear when the toggle is [true]
                if let Some(keybinding) = maybe_shortcut {
                    let span_with_shortcut =
                        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "with_shortcut", %keybinding)
                            .entered();
                    if ui
                        .menu_item_config(name)
                        .shortcut(keybinding.to_string())
                        .build_with_ref(toggle)
                    {
                        // Don't need to toggle manually since it's handled by ImGui (we passed in a mut ref to the variable)
                        debug!(
                            target: UI_DEBUG_USER_INTERACTION,
                            "clicked menu item '{}', value: {}",
                            name,
                            *toggle
                        );
                    } else {
                        trace!(target: UI_TRACE_USER_INPUT, "menu item not toggled via ui");
                    }

                    let key_pressed = ui.is_key_index_pressed_no_repeat(keybinding.shortcut as i32);
                    let modifiers_pressed = keybinding.all_modifiers_held(ui);
                    trace!(target: UI_TRACE_USER_INPUT, ?key_pressed, ?modifiers_pressed);
                    if key_pressed && modifiers_pressed {
                        *toggle ^= true;
                        debug!(
                            target: UI_DEBUG_USER_INTERACTION,
                            "keypress for menu item '{}', value: {}",
                            name,
                            *toggle
                        );
                    }

                    span_with_shortcut.exit();
                } else {
                    let span_no_shortcut =
                        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "no_shortcut").entered();
                    if ui
                        .menu_item_config(name)
                        .shortcut(NO_SHORTCUT)
                        .build_with_ref(toggle)
                    {
                        debug!(
                            target: UI_DEBUG_USER_INTERACTION,
                            "clicked menu item '{}', toggled => {}",
                            name,
                            *toggle
                        );
                    } else {
                        trace!(target: UI_TRACE_USER_INPUT, "not toggled via ui");
                    }

                    span_no_shortcut.exit();
                }
                span_create_toggle_menu_item.exit();
            }; //end toggle_menu_item

        let main_menu_bar_token = match ui.begin_main_menu_bar() {
            None => {
                //Menu bar isn't visibliie
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    "main menu bar not visible (should always be visible)"
                );
                return Ok(()); //Skip drawing the main menu bar
            }
            Some(token) => token,
        };

        fn menu<F>(ui: &imgui::Ui, label: &str, func: F) -> FallibleFn
        where
            F: FnOnce() -> FallibleFn,
        {
            trace_span!(target: UI_TRACE_BUILD_INTERFACE, "menu", menu_label = label).in_scope(
                || {
                    let mut result = Ok(());
                    match ui.begin_menu(label) {
                        None => {
                            trace!(target: UI_TRACE_BUILD_INTERFACE, "menu not visible");
                        }
                        Some(token) => {
                            result = func();
                            token.end();
                        }
                    }
                    result
                },
            )
        }

        menu(ui, "Tools", || {
            toggle_menu_item(
                "Metrics",
                show_metrics_window,
                &Some(&keys.toggle_metrics_window),
                indoc! {r"
                    Toggles the metrics window.

                    The Metrics window shows statistics (metrics) about the UI
                "},
            );
            toggle_menu_item(
                "Demo Window",
                show_demo_window,
                &Some(&keys.toggle_demo_window),
                indoc! {r"
                    Toggles the ImGUI demo window
                "},
            );
            toggle_menu_item(
                "UI Management",
                show_ui_management_window,
                &Some(&keys.toggle_ui_managers_window),
                indoc! {r"
                    Toggles the UI management window.

                    The UI management window allows you to modify the UI, such as changing the font.
                "},
            );
            toggle_menu_item(
                "Config",
                show_config_window,
                &Some(&keys.toggle_config_window),
                indoc! {r"
                    Toggles the Configuration Options window.
                "},
            );

            // Semi-hacky quit handling
            // Makes a toggle and if it's set to true, sends quit message to program
            let mut exit = false;
            toggle_menu_item(
                "Exit",
                &mut exit, // Doesn't show any checkboxes or anything
                &Some(&keys.exit_app),
                indoc! {r"
                    Exits the application. Exactly the same as clicking the close button
                "},
            );
            if exit {
                debug!(
                    target: UI_DEBUG_USER_INTERACTION,
                    "user clicked quit menu item, sending quit signals"
                );
                send_message(
                    Program(QuitAppNoError(QuitInteractionByUser)),
                    message_sender,
                )?;
                debug!(target: UI_DEBUG_GENERAL, "ui should quit soon");
            }
            FallibleFn::Ok(())
        })?; // end tools

        main_menu_bar_token.end();
        FallibleFn::Ok(())
    })?; // end main_menu_bar

    if *show_demo_window {
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "show_demo_window")
            .in_scope(|| ui.show_demo_window(show_demo_window));
    } else {
        trace!(target: UI_TRACE_BUILD_INTERFACE, "demo window hidden");
    }
    if *show_metrics_window {
        trace_span!(target: UI_TRACE_BUILD_INTERFACE, "show_metrics_window")
            .in_scope(|| ui.show_metrics_window(show_metrics_window));
    } else {
        trace!(target: UI_TRACE_BUILD_INTERFACE, "metrics window hidden");
    }
    build_window("UI Management", managers, show_ui_management_window, ui)?;
    build_window_fn("Config", render_config_ui, show_config_window, ui)?;

    span_build_ui.record("elapsed", display(timer));
    span_build_ui.exit();
    trace!(
        target: UI_TRACE_BUILD_INTERFACE,
        "{0} END BUILD FRAME {frame} {0}",
        str::repeat("=", 50),
        frame = ui.frame_count()
    );
    Ok(())
}