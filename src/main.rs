#![warn(missing_docs)]

//!

use std::io;

use clap::*;
use color_eyre::eyre::{self};
use color_eyre::owo_colors::OwoColorize;
use eyre::eyre;
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use tracing::metadata::LevelFilter;
use tracing::*;
use tracing_subscriber::fmt::{format::*, time};
use tracing_subscriber::util::TryInitError;

shadow!(build); //Required for shadow-rs to work

/// Main entrypoint for the program
///
/// Handles the important setup before handing control over to the actual program:
/// * Initialises `eyre` (for panic/error handling)
/// * Initialises `tracing` (for logging)
/// * Processes command-line arguments
/// * Runs the program for real
fn main() -> eyre::Result<()> {
    //Init the important stuff
    init_eyre()?;
    init_tracing()?;
    debug!("[tracing] and [eyre] initialised");

    debug_span!("parse_cli_args").in_scope(|| {
        let args = CliArgs::parse();

        for _ in 0..args.count {
            info!("Hello {}!", args.name)
        }
    });
    //
    // info!("");
    //
    // let _span = info_span!("empty_span").entered();
    // let _span = info_span!("test_span", test_prop = "HOLA").entered();
    // let _span = info_span!("child_span", i_am_a_child = "true").entered();
    // event!(Level::INFO, "something happened");
    // event!(Level::DEBUG, "something happened");
    // event!(Level::ERROR, "something happened");
    // event!(Level::TRACE, "something happened");
    // event!(Level::WARN, "something happened");
    // info!("TEST)");
    //
    // let string = "string";
    // assert_eq!(5, 5, "5 is five");
    // assert_eq!(5, 69, "5 isn't 69");

    return Ok(());
    // return Err(eyre!("Test lol: {} {asd} {string}", 1, asd = 69420));
}

/// CLI Arguments for the program
///
/// Parsed by [clap] in [main]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = )]
#[command()]
struct CliArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

///
fn init_tracing() -> eyre::Result<(), TryInitError> {
    use tracing_error::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    let standard_format = format()
        // .compact()
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
}

fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()
}
