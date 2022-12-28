use crate::helper::logging::event_targets::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct TracingConfig {
    /// Controls how errors are logged in the app
    ///
    /// For a demo/example, see the [color_eyre::eyre::Report] documentation
    pub error_style: ErrorLogStyle,

    /// Vec of log filters, that control what log targets will be logged
    ///
    /// By creating a log filter, you can ignore events from certain log targets (such as [UI_SPAMMY])
    ///
    /// Only the first matching filter will be used (the rest will be skipped), and if none match then the event will be logged by default.
    pub target_filters: Vec<LogTargetFilter>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            error_style: ErrorLogStyle::ShortWithCause,
            target_filters: vec![
                //Standard, these are almost always unnecessary
                // Most of these are here just-in-case, or for profiling (like inferno/[tracing-flame])
                LogTargetFilter::new(UI_TRACE_EVENT_LOOP, false),
                LogTargetFilter::new(UI_TRACE_BUILD_INTERFACE, false),
                LogTargetFilter::new(UI_TRACE_RENDER, false),
                LogTargetFilter::new(THREAD_TRACE_MESSAGE_LOOP, false),
                LogTargetFilter::new(THREAD_TRACE_MUTEX_SYNC, false),
                LogTargetFilter::new(DATA_DEBUG_DUMP_OBJECT, false),
                LogTargetFilter::new(PROGRAM_TRACE_GLOBAL_LOOP, false),
                LogTargetFilter::new(ENGINE_TRACE_GLOBAL_LOOP, false),
                LogTargetFilter::new(THREAD_TRACE_MESSAGE_IGNORED, false),
                LogTargetFilter::new(PROGRAM_TRACE_THREAD_STATUS_POLL, false),
            ],
        }
    }
}

/// Holds a regex that matches on an event's target, and a [bool] that indicates whether that target should be enabled or disabled
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LogTargetFilter {
    pub target: &'static str,
    pub enabled: bool,
}

impl LogTargetFilter {
    /// Creates a filter that matches if the target starts with a specified string. The input can be regex
    /// Creates a new filter from a regex string
    pub fn new(val: &'static str, enabled: bool) -> LogTargetFilter {
        LogTargetFilter {
            target: val,
            enabled,
        }
    }
}

/// Enum that controls how errors (color_eyre::eyre::Report) are formatted
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ErrorLogStyle {
    Short,
    ShortWithCause,
    WithBacktrace,
    Debug,
}
