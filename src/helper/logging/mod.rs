use std::error::Error;

use color_eyre::{Help, Report};
use tracing::{error, warn};

use crate::config::tracing_config::{ErrorLogStyle, ERROR_LOG_STYLE};

pub mod event_targets;

/// Logs an expression's string representation and returns the original expression. The format string can also be customised in the second overload (with custom arguments)
///
/// ### Examples:
///
/// `let calculation = log_expr!(do_maths())` prints ```run `do_maths()` ``` and returns whatever value `do_maths()` returned (the expression is directly injected into the generated code, so the expression can return nothing).
/// This form simply calls [log_expr] with `$expression_name=expr` and ```$format_and_args="run `{expr}` ```
///
/// ```let add_two_numbers = log_expr!(f64::from(5+5) * 3.21f64, "Adding numbers: {expr}");```
///
/// prints
///
/// ```Adding numbers: f64::from(5 + 5) * 3.21f64```
#[macro_export]
macro_rules! log_expr {
    ($expression:expr) => {
        log_expr!($expression, expr, "run `{expr}`")
    };
    ($expression:expr, $format_and_args:tt) => {{
        let value = $expression;
        tracing::trace!($format_and_args, expr = stringify!($expression));
        value
    }};
}

#[macro_export]
macro_rules! log_variable {
    ($variable:ident) => {
        tracing::trace!("{}={}", stringify!($variable), $variable);
    };
    ($variable:ident:?) => {
        tracing::trace!("{}={:?}", stringify!($variable), $variable);
    };
    ($variable:ident:#?) => {
        tracing::trace!("{}={:#?}", stringify!($variable), $variable);
    };
}

/// Same as [log_expr] but includes the value of the evaluated expression
///
/// When using a custom format and arguments, the expression is `expr` and the value is `val`
#[macro_export]
macro_rules! log_expr_val {
    ($expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val}")
    };
    (?$expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val:?}")
    };
    (#?$expression:expr) => {
        log_expr_val!($expression, "eval `{expr}` => {val:#?}")
    };
    ($expression:expr, $format_and_args:tt) => {{
        let val = $expression;
        tracing::trace!($format_and_args, expr = stringify!($expression), val = val);
        val
    }};
}

/// Function that logs an error in whichever way the app is configured to log errors
pub fn log_error(report: &Report) {
    match ERROR_LOG_STYLE {
        ErrorLogStyle::Short => error!("{}", report),
        ErrorLogStyle::ShortWithCause => error!("{:#}", report),
        ErrorLogStyle::WithBacktrace => error!("{:?}", report),
        ErrorLogStyle::Debug => error!("{:#?}", report),
    }
}

/// Function that logs an error in whichever way the app is configured to log errors, but at the level of [tracing::warn]
pub fn log_error_as_warning(report: &Report) {
    match ERROR_LOG_STYLE {
        ErrorLogStyle::Short => warn!("{}", report),
        ErrorLogStyle::ShortWithCause => warn!("{:#}", report),
        ErrorLogStyle::WithBacktrace => warn!("{:?}", report),
        ErrorLogStyle::Debug => warn!("{:#?}", report),
    }
}

/// Function to convert a boxed error (`&Box<dyn Error>`) to an owned [Report]
pub fn dyn_error_to_report(error: &Box<dyn Error>) -> Report {
    let formatted_error = match ERROR_LOG_STYLE {
        ErrorLogStyle::Short => {
            format!("{error:}")
        }
        ErrorLogStyle::ShortWithCause => {
            format!("{error:#}")
        }
        ErrorLogStyle::WithBacktrace => {
            format!("{error:?}")
        }
        ErrorLogStyle::Debug => {
            format!("{error:#?}")
        }
    };
    return Report::msg(formatted_error)
        .note("this error was converted from a `&Box<dyn Error>`, information may be missing and/or incorrect");
}
