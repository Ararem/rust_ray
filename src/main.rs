use std::io;

use color_eyre::eyre::{self};
use color_eyre::owo_colors::OwoColorize;
use eyre::eyre;
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use tracing::*;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{format, time};
use tracing_subscriber::util::SubscriberInitExt;

shadow!(build); //Required for shadow-rs to work

#[instrument]
fn main() -> eyre::Result<()> {
    //Init the important stuff
    init_eyre()?;
    init_tracing()?;

    let _span = info_span!("test_span").entered();
    let _span = info_span!("child_span").entered();
    event!(Level::INFO, "something happened");
    info!("TEST)");

    let string = "string";
    assert_eq!(5, 5, "5 is five");
    assert_eq!(5, 69, "5 isn't 69");

    // return Ok(());
    return Err(eyre!("Test lol: {} {asd} {string}", 1, asd = 69420));
}

/// Tracing filter.
///
/// Can be any of "error", "warn", "info", "debug", or
/// "trace". Supports more granular filtering, as well; see documentation for
/// [`tracing_subscriber::EnvFilter`][EnvFilter].
///
/// [EnvFilter]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/struct.EnvFilter.html
fn init_tracing() -> eyre::Result<()> {
    use tracing_error::*;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::*;

    let standard_layer = fmt::layer()
        .event_format(format().compact())
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_span_events(FmtSpan::ACTIVE)
        .log_internal_errors(true)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        // .with_timer(time())
        .with_writer(io::stdout)
        // .with_test_writer()
        ;

    let error_layer = ErrorLayer::default();

    tracing_subscriber::registry()
        .with(standard_layer)
        .with(error_layer)
        .init();

    Ok(())
}

fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()?;

    Ok(())
}
