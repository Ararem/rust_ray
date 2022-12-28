#![warn(missing_docs)]
#![warn(clippy::all)]

//! # A little test raytracer project
use std::io;

use crate::config::read_config_value;
use color_eyre::eyre;
use tracing::level_filters::LevelFilter;
use tracing::*;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt::format;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::time::uptime;

use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_error;

mod build;
mod config;
mod engine;
mod helper;
mod program;
mod resources;
mod ui;

pub type FallibleFn = eyre::Result<()>;

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises [eyre] (for panic/error handling)
/// * Initialises [tracing] (for logging)
/// * TODO: Processes command-line arguments
/// * Runs the [program] for real
fn main() -> FallibleFn {
    init_eyre()?;
    init_tracing()?;

    helper::panic_pill::red_or_blue_pill();

    debug!(
        target: MAIN_DEBUG_GENERAL,
        "initialised [tracing] and [eyre], skipped cli args"
    );

    let args = std::env::args();
    let args_os = std::env::args_os();
    debug!(target: MAIN_DEBUG_GENERAL, ?args, ?args_os, "command line");
    debug!(target: MAIN_DEBUG_GENERAL, "core init done");

    info!(target: PROGRAM_INFO_LIFECYCLE, "starting program");
    match program::run() {
        Ok(program_return_value) => {
            info!(
                target: PROGRAM_INFO_LIFECYCLE,
                ?program_return_value,
                "program completed successfully"
            );
            info!(target: PROGRAM_INFO_LIFECYCLE, "goodbye :)");
            Ok(program_return_value)
        }
        Err(report) => {
            let formatted_error = format_error(&report);
            error!(
                target: PROGRAM_INFO_LIFECYCLE,
                formatted_error, "program exited unsuccessfully"
            );
            info!(target: PROGRAM_INFO_LIFECYCLE, "goodbye :(");
            Err(report)
        }
    }
}

/// Initialises [eyre]. Called as part of the core init
fn init_eyre() -> FallibleFn {
    color_eyre::install()
}

/// Initialises the [tracing] system. Called as part of the core init
fn init_tracing() -> FallibleFn {
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    let standard_format = format()
        .compact()
        .with_ansi(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_target(false)
        .with_level(true)
        .with_timer(uptime())
        .with_source_location(false)
        .with_level(true);

    let standard_layer = fmt::layer()
        .with_span_events(FmtSpan::ACTIVE)
        .log_internal_errors(true)
        .event_format(standard_format)
        .with_writer(io::stdout)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy(),
        )
        .with_filter(FilterFn::new(|meta| {
            let target = meta.target();

            match target {
                // If we encounter an error with the config, then we may try logging a warning while filtering a previous message
                // This would recurse, so bypass and exit early if the target matches the warning/error targets
                GENERAL_WARNING_NON_FATAL
                | GENERAL_ERROR_FATAL
                | REALLY_FUCKING_BAD_UNREACHABLE
                | DOMINO_EFFECT_FAILURE => true,
                // Otherwise (default), scan the config
                _ => {
                    let configured_targets = read_config_value(|config| &config.runtime.tracing.target_filters);
                    for filter in configured_targets {
                        if filter.target == target {
                            return filter.enabled;
                        }
                    }
                    true
                }
            }
        }));

    tracing_subscriber::registry()
        .with(standard_layer)
        // .with(tracing_flame::FlameLayer::with_file("./tracing.folded").unwrap().0)
        .try_init()?;

    Ok(())
}
