use std::any::Any;
use std::error::Error;

use color_eyre::eyre::format_err;
use color_eyre::{Help, Report};
use tracing::field::{Field, Visit};
use tracing::{error, warn, Value};

use crate::config::tracing_config::{ErrorLogStyle, ERROR_LOG_STYLE};

pub(crate) mod event_targets;

/// Function that logs an error in whichever way the app is configured to log errors
pub fn format_error(report: &Report) -> &str {
    match ERROR_LOG_STYLE {
        ErrorLogStyle::Short => format!("{}", report),
        ErrorLogStyle::ShortWithCause => format!("{:#}", report),
        ErrorLogStyle::WithBacktrace => format!("{:?}", report),
        ErrorLogStyle::Debug => format!("{:#?}", report),
    }
    .as_str()
}
//noinspection DuplicatedCode - Duped the `match ERROR_LOG_STYLE` part
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

//noinspection DuplicatedCode - Duped the `match ERROR_LOG_STYLE` part
/// Function to convert a boxed panic error (`&Box<dyn Any + Send>`) to an owned [Report]
pub fn dyn_panic_to_report(error: &Box<dyn Any + Send>) -> Report {
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
        .note("this error was converted from a `&Box<dyn Any+Send>`, information may be missing and/or incorrect");
}
