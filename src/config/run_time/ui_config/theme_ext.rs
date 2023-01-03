use crate::config::run_time::ui_config::theme::{Colour, Theme};
use tracing::Level;

pub fn colour_for_tracing_level(colours: &Theme, level: &Level) -> Colour {
    match *level {
        Level::TRACE => colours.value.level_trace,
        Level::DEBUG => colours.value.level_debug,
        Level::INFO => colours.value.level_info,
        Level::WARN => colours.value.level_warn,
        Level::ERROR => colours.value.level_error,
    }
}