use color_eyre::eyre;
use std::io;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{format, time};

/// Logs an expression, then inserts the expression into the block so that the value can still be used
#[macro_export] macro_rules! log_expr {
    ($e:expr) => {
        {
            tracing::trace!(stringify!($e));
            $e
        }
    };
}

pub fn init_tracing() -> eyre::Result<()> {
    use tracing_error::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};

    //This is all simple config stuff, not much to explain
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
        .try_init()?;

    Ok(())
}
