//! Contains compile-time configuration options

/// Macro that generates a config flag with a given [name] and [value]. Field will be `static` to ensure no "expression always true" warnings happen
macro_rules! flag {
    ($name:ident, $value:expr, $documentation:literal) => {
        #[doc=$documentation]
        pub static $name: bool = $value;
    };
}
macro_rules! string {
    ($name:ident, $value:expr, $documentation:literal) => {
        #[doc=$documentation]
        pub static $name: bool = $value;
    };
}
// /// Macro that generates a config flag with a given [name] and [value]. Same as [flag] but generates a `const` field not a `static` one
// macro_rules! const_flag {
//     ($name:ident, $value:expr) => {pub const $name:bool = $value;};
// }

pub mod tracing {
    use lazy_static::lazy_static;
    use regex::Regex;
    use super::super::helper::logging::event_targets::*;

    /// Holds a regex that matches on an event's target, and a [bool] that indicates whether that target should be enabled or disabled
    pub struct LogTargetFilter {
        pub regex: Regex,
        pub enabled: bool,
    }
    impl LogTargetFilter {
        /// Creates a filter that matches if the target starts with a specified string. The input can be regex
        pub fn starts_with(start: &str, enabled: bool) -> LogTargetFilter {
            LogTargetFilter::new(format!("{}.*", start).as_str(), enabled)
        }
        /// Creates a new filter from a regex string
        pub fn new(regex: &str, enabled: bool) -> LogTargetFilter {
            LogTargetFilter {
                regex: Regex::new(regex).expect("regex failed to parse"),
                enabled,
            }
        }
    }
    lazy_static! {
        /// Vec of log filters. The first matching filter will affect if the event is logged or not, and if no match then the event will be logged.
        pub static ref LOG_FILTERS: Vec<LogTargetFilter> = vec![
            // LogTargetFilter::starts_with(UI_SPAMMY, false),
        ];
    }
}
