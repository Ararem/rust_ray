use crate::config::run_time::ui_config::theme::{Colour, Theme};
use tracing::Level;

impl Theme {
    pub fn colour_for_tracing_level(&self, level: &Level) -> Colour {
        match *level {
            Level::TRACE => self.value.level_trace,
            Level::DEBUG => self.value.level_debug,
            Level::INFO => self.value.level_info,
            Level::WARN => self.value.level_warn,
            Level::ERROR => self.value.level_error,
        }
    }
}
