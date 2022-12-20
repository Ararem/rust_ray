#![warn(missing_docs)]
#![warn(clippy::all)]

//! # A little test raytracer project
use std::io;

use color_eyre::eyre;
use tracing::level_filters::LevelFilter;
use tracing::*;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::config::tracing_config::STANDARD_FORMAT;
use crate::helper::logging::event_targets::*;
use crate::helper::logging::format_error;

mod build;
mod config;
mod engine;
mod helper;
mod program;
mod resources;
mod ui;

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises [eyre] (for panic/error handling)
/// * Initialises [tracing] (for logging)
/// * TODO: Processes command-line arguments
/// * Runs the [program] for real
fn main() -> eyre::Result<()> {
    init_eyre()?;
    init_tracing()?;
    helper::panic_pill::red_or_blue_pill();

    debug!(target: MAIN_DEBUG_GENERAL, "initialised [tracing] and [eyre], skipped cli args");

    let args = std::env::args();
    let args_os = std::env::args_os();
    debug!(target: MAIN_DEBUG_GENERAL, ?args, ?args_os, "command line");
    debug!(target: MAIN_DEBUG_GENERAL, "core init done");

    info!(target: PROGRAM_INFO_LIFECYCLE, "starting program");
    return match program::run() {
        Ok(program_return_value) => {
            info!(target: PROGRAM_INFO_LIFECYCLE, ?program_return_value, "program completed successfully");
            info!(target: PROGRAM_INFO_LIFECYCLE, "goodbye :)");
            Ok(program_return_value)
        }
        Err(report) => {
            let formatted_error = format_error(&report);
            error!(target: PROGRAM_INFO_LIFECYCLE, formatted_error, "program exited unsuccessfully");
            info!(target: PROGRAM_INFO_LIFECYCLE, "goodbye :(");
            Err(report)
        }
    };
}

/// Initialises [eyre]. Called as part of the core init
pub fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()
}

/// Initialises the [tracing] system. Called as part of the core init
fn init_tracing() -> eyre::Result<()> {
    use tracing_error::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    let standard_layer = fmt::layer()
        .with_span_events(FmtSpan::NONE)
        .log_internal_errors(true)
        .event_format(STANDARD_FORMAT.clone())
        .with_writer(io::stdout)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy(),
        )
        .with_filter(FilterFn::new(|meta| {
            let target = meta.target();
            for filter in config::tracing_config::LOG_FILTERS.iter() {
                if filter.regex.is_match(target) {
                    return filter.enabled;
                }
            }
            return true;
        }));

    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(standard_layer)
        .with(error_layer)
        .try_init()?;

    Ok(())
}
