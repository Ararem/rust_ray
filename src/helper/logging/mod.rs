use std::any::Any;
use std::error::Error;

use crate::config::run_time::tracing_config::ErrorLogStyle;
use crate::config::run_time::RuntimeAppConfig;
use crate::config::Config;
use color_eyre::{Help, Report};
use indoc::formatdoc;
use tracing::field::{display, DisplayValue};

use crate::FallibleFn;

pub mod event_targets;
pub mod span_time_elapsed_field;

/// Function that logs an error in whichever way the app is configured to log errors
pub fn format_error(report: &Report, config: Config) -> DisplayValue<String> {
    display(format_error_string(report, config))
}
pub fn format_error_string(report: &Report, config: Config) -> String {
    match config.runtime.tracing.error_style {
        ErrorLogStyle::Short => format!("{}", report),
        ErrorLogStyle::ShortWithCause => format!("{:#}", report),
        ErrorLogStyle::WithBacktrace => format!("{:?}", report),
        ErrorLogStyle::Debug => format!("{:#?}", report),
    }
}

/// Function to convert a boxed error (`&Box<dyn Error>`) to an owned [Report]
#[allow(clippy::borrowed_box)] // Can't do it because it's a dyn Trait, also needs this signature for compat reasons
pub fn dyn_error_to_report(error: &Box<dyn Error>, config: Config) -> Report {
    let formatted_error = match config.runtime.tracing.error_style {
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
    Report::msg(formatted_error)
        .note("this error was converted from a `&Box<dyn Error>`, information may be missing and/or incorrect")
}

/// Function to convert a boxed panic error (`&Box<dyn Any + Send>`) to an owned [Report]
pub fn dyn_panic_to_report(boxed_error: &Box<dyn Any + Send>, config: Config) -> Report {
    // Default case
    let mut formatted_error = formatdoc! {r"
        <unable to convert panic, does not implement any known types>
     "};
    macro_rules! case {
        ($( &$type:ty )| *, $type_str:ident, $val:ident, $case:expr) => {$(
            //When the [Box] contains an object T -> &T
            if let Some($val) = (&**boxed_error).downcast_ref::<$type>() {
                #[allow(unused)]
                let $type_str = stringify!($type);
                formatted_error = $case;
            }
            //When the [Box] contains a reference &T -> &&T
            else if let Some(&$val) = (&**boxed_error).downcast_ref::<&$type>() {
                #[allow(unused)]
                let $type_str = stringify!($type);
                formatted_error = $case;
            }
        )+};
    }
    // Primitive types
    case! {
        &i8 | &i16 | &i32 | &i64 | &i128
        |&u8 | &u16 | &u32 | &u64 | &u128
        |&f32 | &f64
        |&bool
        |&char | &String /*&str | str isn't sized so can't use it here, have to impl separately*/
        |&usize | &isize
        , type_name, val, {
        format!("[{type_name}]: {val}")
    }}
    // Special cases
    case! {
    &(), type_name, _val, {
        format!("[{type_name}]: ()")
    }}
    case! {
       &Report, type_name, val, {
           format!("[{type_name}]: {}", format_error(val, config))
    }}
    case! {
       &FallibleFn, type_name, val, {
           match val { Ok(()) => format!("[{type_name}]: ()"), Err(report) => format!("[{type_name}]: {}", format_error(report, config)) }
    }}
    // Special case since [str] is dynamically sized
    if let Some(val) = (**boxed_error).downcast_ref::<&str>() {
        formatted_error = format!("[str]: {}", *val);
    }
    Report::msg(formatted_error)
        .note("this error was converted from a `&Box<dyn Any+Send>`, information may be missing and/or incorrect")
}
