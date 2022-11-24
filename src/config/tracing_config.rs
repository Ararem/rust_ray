use super::super::helper::logging::event_targets::*;
use lazy_static::lazy_static;
use regex::Regex;
use tracing_subscriber::{fmt::format::*, fmt::time::*};

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

    /// Vec of log filters, that control what log targets will be logged
    ///
    /// By creating a log filter, you can ignore events from certain log targets (such as [UI_SPAMMY])
    ///
    /// Only the first matching filter will be used (the rest will be skipped), and if none match then the event will be logged by default.
    pub static ref LOG_FILTERS: Vec<LogTargetFilter> = vec![
        LogTargetFilter::starts_with(UI_SPAMMY, false),
    ];

    /// Standard format for tracing events
    pub static ref STANDARD_FORMAT:  Format<Compact, Uptime> = format()
        .compact()
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_level(true)
        .with_timer(uptime())
        .with_source_location(false)
        .with_level(true);
}
