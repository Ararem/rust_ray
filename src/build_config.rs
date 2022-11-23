//! Contains compile-time configuration options

/// Macro that generates a config flag with a given [name] and [value]. Field will be `static` to ensure no "expression always true" warnings happen
macro_rules! flag {
    ($name:ident, $value:expr, $documentation:literal) => {
        #[doc=$documentation]
        pub static $name: bool = $value;
    };
}
// /// Macro that generates a config flag with a given [name] and [value]. Same as [flag] but generates a `const` field not a `static` one
// macro_rules! const_flag {
//     ($name:ident, $value:expr) => {pub const $name:bool = $value;};
// }

pub(crate) mod tracing {
    use lazy_static::lazy_static;
    use regex::Regex;
    use super::super::helper::event_targets::*;

    flag!(ENABLE_UI_TRACE, false, r"Flag to enable UI trace logging. ");

    pub(crate) struct LogFilter {
        pub(crate) regex: Regex,
        pub(crate) enabled: bool,
    }
    impl LogFilter {
        pub fn starts_with(start: &str, enabled: bool) -> LogFilter {
            LogFilter::new(format!("{}.*", start).as_str(), enabled)
        }
        pub fn new(regex: &str, enabled: bool) -> LogFilter {
            LogFilter {
                regex: Regex::new(regex).expect("regex failed to parse"),
                enabled,
            }
        }
    }
    lazy_static! {
        pub(crate) static ref LOG_FILTERS: Vec<LogFilter> = vec![
            LogFilter::starts_with(UI_SPAMMY, false),
            // LogFilter::starts_with(PROGRAM_RENDER, false)
        ];
    }
}
