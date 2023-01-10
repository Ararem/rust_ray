mod config_ui_impl;
mod shared;
mod ui_management;

use crate::config::read_config_value;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::span_time_elapsed_field::SpanTimeElapsedField;
use crate::program::thread_messages::ProgramThreadMessage::QuitAppNoError;
use crate::program::thread_messages::QuitAppNoErrorReason::QuitInteractionByUser;
use crate::program::thread_messages::ThreadMessage::Program;
use crate::program::thread_messages::*;
use crate::ui::ui_data::UiData;
use crate::ui::ui_system::UiManagers;
use crate::FallibleFn;
use config_ui_impl::render_config_ui;
use indoc::indoc;
use multiqueue2::{BroadcastReceiver, BroadcastSender};
use tracing::field::*;
use tracing::*;
use shared::input::handle_shortcut;
use shared::menu_utils::{menu, toggle_menu_item};
use shared::window_utils::{build_window, build_window_fn};
use crate::ui::build_ui_impl::shared::error_display::render_errors_popup;

pub trait UiItem {
    fn render(&mut self, ui: &imgui::Ui, visible: bool) -> FallibleFn;
}

pub(super) fn build_ui(
    ui: &imgui::Ui,
    managers: &mut UiManagers,
    data: &mut UiData,
    message_sender: &BroadcastSender<ThreadMessage>,
    _message_receiver: &BroadcastReceiver<ThreadMessage>,
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

    // refs to reduce clutter
    let show_demo_window = &mut data.windows.show_demo_window;
    let show_metrics_window = &mut data.windows.show_metrics_window;
    let show_ui_management_window = &mut data.windows.show_ui_management_window;
    let show_config_window = &mut data.windows.show_config_window;
    let keys = read_config_value(|config| config.runtime.keybindings);

    trace_span!(target: UI_TRACE_BUILD_INTERFACE, "main_menu_bar").in_scope(|| {
        let main_menu_bar_token = match ui.begin_main_menu_bar() {
            None => {
                //Menu bar isn't visible
                warn!(
                    target: GENERAL_WARNING_NON_FATAL,
                    "main menu bar not visible (should always be visible)"
                );
                return Ok(()); //Skip drawing the main menu bar
            }
            Some(token) => token,
        };
        trace!(target: UI_TRACE_BUILD_INTERFACE, "building main menu bar");

        menu(ui, "Tools", || {
            toggle_menu_item(
                ui,
                "Demo Window",
                show_demo_window,
                &keys.toggle_demo_window.to_string(),
                indoc! {r"
                Toggles the ImGUI demo window.

                The demo window demonstrates the features of Dear ImGUI, and provides some debugging tools for debugging ImGUI
            "},
            )?;
            toggle_menu_item(
                ui,
                "Metrics",
                show_metrics_window,
                &keys.toggle_metrics_window.to_string(),
                indoc! {r"
                Toggles the ImGUI metrics window.

                The metrics window shows statistics and metrics about Dear ImGUI
            "},
            )?;
            toggle_menu_item(
                ui,
                "Config",
                show_config_window,
                &keys.toggle_config_window.to_string(),
                indoc! {r"
                Shows/hides the config window.

                The config window allows modifying the app configuration. Very much WIP
            "},
            )?;
            toggle_menu_item(
                ui,
                "UI Management",
                show_ui_management_window,
                &keys.toggle_ui_managers_window.to_string(),
                indoc! {r"
                    Toggles the UI management window.

                    The UI management window allows you to control the UI, such as changing the font.
            "},
            )?;

            // Semi-hacky quit handling
            // Makes a toggle and if it's set to true, sends quit message to program
            let mut exit = false;
            toggle_menu_item(
                ui,
                "Exit",
                &mut exit, // Doesn't show any checkboxes or anything
                &keys.exit_app.to_string(),
                indoc! {r"
                    Exits the application.

                    Exactly the same as clicking the close button
                "},
            )?;
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

            Ok(())
        })?; //end Tools menu

        main_menu_bar_token.end();
        FallibleFn::Ok(())
    })?; // end main menu

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
    render_errors_popup(ui);

    trace_span!(target: UI_TRACE_USER_INPUT, "handle_input").in_scope(|| {
        handle_shortcut(
            ui,
            "show demo window",
            &keys.toggle_demo_window,
            show_demo_window,
        );
        handle_shortcut(
            ui,
            "show config window",
            &keys.toggle_config_window,
            show_config_window,
        );
        handle_shortcut(
            ui,
            "show ui management window",
            &keys.toggle_ui_managers_window,
            show_ui_management_window,
        );
        handle_shortcut(
            ui,
            "show metrics window",
            &keys.toggle_metrics_window,
            show_metrics_window,
        );
    });

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
