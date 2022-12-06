#![warn(missing_docs)]

//! # A little test raytracer project
use std::io;
use std::process::ExitCode;

use color_eyre::{eyre, Report};
use tracing::*;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::config::tracing_config::STANDARD_FORMAT;

mod program;
mod config;
mod helper;
mod resources;
mod build;
mod ui;
mod engine;

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises [eyre] (for panic/error handling)
/// * Initialises [tracing] (for logging)
/// * TODO: Processes command-line arguments
/// * Runs the [program] for real
fn main() -> eyre::Result<ExitCode> {
    init_eyre()?;
    init_tracing()?;
    debug!("initialised [tracing] and [eyre]");

    debug!("skipping CLI and Env args");

    //Event loop
    debug!("main init complete, starting");

    trace!("running program");
    return match program::run() {
        Ok(_) => {
            info!("program completed successfully");
            info!("goodbye");
            Ok(ExitCode::SUCCESS)
        }
        Err(report) => {
            warn!("program completed with errors");
            Err(report)
        }
    }
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
        .with_span_events(FmtSpan::ACTIVE)
        .log_internal_errors(true)
        .event_format(STANDARD_FORMAT.clone())
        .with_writer(io::stdout)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::TRACE.into())
                .from_env_lossy()
        )
        .with_filter(
            FilterFn::new(|meta| {
                let target = meta.target();
                for filter in config::tracing_config::LOG_FILTERS.iter() {
                    if filter.regex.is_match(target) {
                        return filter.enabled
                    }
                }
                return true;
            })
        );

    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(standard_layer)
        .with(error_layer)
        .try_init()?;

    Ok(())
}
