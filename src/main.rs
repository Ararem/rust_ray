#![warn(missing_docs)]

//!

use std::io;

use color_eyre::eyre::{self};
use eyre::eyre;
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use tracing::metadata::LevelFilter;
use tracing::*;
use tracing_subscriber::fmt::{format::*, time};
use tracing_subscriber::util::TryInitError;

shadow!(build); //Required for shadow-rs to work

fn main() -> eyre::Result<()> {
    //Init the important stuff
    init_eyre()?;
    init_tracing()?;

    let _span = info_span!("empty_span").entered();
    let _span = info_span!("test_span", test_prop = "HOLA").entered();
    let _span = info_span!("child_span", i_am_a_child = "true").entered();
    event!(Level::INFO, "something happened");
    event!(Level::DEBUG, "something happened");
    event!(Level::ERROR, "something happened");
    event!(Level::TRACE, "something happened");
    event!(Level::WARN, "something happened");
    info!("TEST)");

    let string = "string";
    assert_eq!(5, 5, "5 is five");
    assert_eq!(5, 69, "5 isn't 69");

    // return Ok(());
    return Err(eyre!("Test lol: {} {asd} {string}", 1, asd = 69420));
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
