#![warn(missing_docs)]

//! A little test raytracer project

mod core;

use std::io;
use std::process::Termination;

use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use tracing::metadata::LevelFilter;
use tracing::*;
use tracing_subscriber::{
    fmt::{format::*, time},
    util::TryInitError,
};

shadow!(build); //Required for shadow-rs to work

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises `eyre` (for panic/error handling)
/// * Initialises `tracing` (for logging)
/// * Processes command-line arguments
/// * Runs the program for real
fn main() -> eyre::Result<()> {
    init_eyre()?;
    init_tracing()?;
    debug!("[tracing] and [eyre] initialised");

    debug!("Skipping CLI args");

    info!("Running program");
    core::run_program()?;
    info!("Ran to completion");

    info!("goodbye");
    return Ok(());
}
///
fn init_tracing() -> eyre::Result<()> {
    use tracing_error::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    let standard_format = format()
        .compact()
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_level(true)
        .with_timer(time::time())
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
                .from_env_lossy()
        )
        // .with_test_writer()
        // .with_timer(time())
        ;

    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(standard_layer)
        .with(error_layer)
        .try_init()
        .wrap_err("[tracing_subscriber::registry] failed to init")
}

fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()
}
