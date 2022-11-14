#![warn(missing_docs)]

//! A little test raytracer project
mod boilerplate;
mod engine;
mod program;

use crate::boilerplate::error_handling::init_eyre;
use crate::boilerplate::logging::init_tracing;
use crate::boilerplate::ui_system::{init_imgui, UiConfig};
use color_eyre::eyre::eyre;
use color_eyre::{eyre};
use glium::glutin::platform::run_return::EventLoopExtRunReturn;
use shadow_rs::shadow;
use std::process::ExitCode;
use std::time::Instant;
use glium::glutin;
use glium::glutin::event_loop::{ControlFlow, EventLoopWindowTarget};
use tracing::*;

shadow!(build); //Required for shadow-rs to work

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises `eyre` (for panic/error handling)
/// * Initialises `tracing` (for logging)
/// * Processes command-line arguments
/// * Runs the program for real
fn main() -> eyre::Result<ExitCode> {
    init_eyre()?;
    init_tracing()?;
    debug!("[tracing] and [eyre] initialised");

    debug!("Skipping CLI and Env args");

    let mut ui_system = init_imgui(
        std::format!(
            "{} v{} - {}",
            build::PROJECT_NAME,
            build::PKG_VERSION,
            build::BUILD_TARGET
        )
        .as_str(),
        UiConfig {
            vsync: true,
            hardware_acceleration: Some(true),
        },
    )?;

    //Event loop
    let program = program::Program { test: true };
    let mut last_frame = Instant::now();

    let run_span = info_span!("run");
    let imgui_internal_span = debug_span!("imgui_internal");
    let _run_span_entered = run_span.enter();
    let _imgui_internal_span_entered = imgui_internal_span.enter();

    let exit_code:i32 = ui_system
        .event_loop
        .run_return(event_loop_run);

    drop(_run_span_entered);

    info!("exit_code: {exit_code}");

    return match exit_code {
        0 => Ok(ExitCode::SUCCESS),
        code => Err(eyre!(
            "run loop returned non-zero exit code {:?}",
            code
        )),
    };
}

fn event_loop_run(event: glium::glutin::event::Event<()>, window_target: &EventLoopWindowTarget<()>, control_flow: &mut ControlFlow){

}