use color_eyre::eyre;
use std::io;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{format, time};

/// Logs an expression's string representation and returns the original expression. The format string can also be customised in the second overload
///
/// ### Examples:
///
/// `let calculation = log_expr!(do_maths())` prints ```run `do_maths()` ``` and returns whatever value `do_maths()` returned (the expression is directly injected into the generated code, so the expression can return nothing).
/// This form simply calls [log_expr] with `$expression_name=expr` and ```$format_and_args="run `{expr}` ```
///
/// ```let add_two_numbers = log_expr!(f64::from(5+5) * 3.21f64, custom_expression_name, "Adding numbers: {custom_expression_name}");```
///
/// prints
///
/// ```Adding numbers: f64::from(5 + 5) * 3.21f64```
#[macro_export] macro_rules! log_expr {
    ($expression:expr) => {
        log_expr!($expression, expr, "run `{expr}`")
    };
    ($expression:expr, $expression_name:ident, $format_and_args:tt) => {
        {
            let value = $expression;
            tracing::trace!($format_and_args, $expression_name=stringify!($expression));
            value
        }
    };
}
#[macro_export] macro_rules! log_expr_val {
    ($expression:expr) => {
        log_expr_val!($expression, expr, val, "eval `{expr}` => {val}")
    };
    ($expression:expr, Debug) => {
        log_expr_val!($expression, expr, val, "eval `{expr}` => {val:?}")
    };
    ($expression:expr, $expression_name:ident, $value_name:ident, $format_and_args:tt) => {
        {
            let $value_name = $expression;
            tracing::trace!($format_and_args, $expression_name=stringify!($expression), $value_name=$value_name);
            $value_name
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
