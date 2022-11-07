use color_eyre::eyre::{self};
use eyre::eyre;
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use shadow_rs::shadow;
use tracing::*;

shadow!(build); //Required for shadow-rs to work

#[instrument]
fn main() -> eyre::Result<()> {
    init_eyre()?;
    init_tracing("trace")?;
    //Same for tracing
    tracing::info_span!("test");
    event!(Level::INFO, "somthing happened");
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
fn init_tracing(filter_directives: &str) -> eyre::Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt::{self, format::FmtSpan};
    use tracing_subscriber::prelude::*;

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_span_events(FmtSpan::ACTIVE);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

fn init_eyre() -> eyre::Result<()> {
    color_eyre::install()?;
    Ok(())
}
