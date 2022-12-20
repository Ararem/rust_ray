use std::panic;

use tracing::{debug, error};

use crate::helper::logging::event_targets::MAIN_DEBUG_GENERAL;
use crate::helper::logging::event_targets::REALLY_FUCKING_BAD_UNREACHABLE;

pub(crate) fn red_or_blue_pill() {
    debug!(
        target: MAIN_DEBUG_GENERAL,
        "generating a red and blue pill, choose wisely"
    );
    // Got panics? We've got a pill for that!
    //
    // Custom panic hook that prints a log message and quits the whole process
    panic::set_hook(Box::new(|panic_info| {
        error!(
            "you chose the red pill, please stand by while process is ejected from OS matrix"
        );
        error!(
            target: REALLY_FUCKING_BAD_UNREACHABLE,
            ?panic_info,
            "process panicked. process will now exit"
        );
        std::process::abort();
    }));
    debug!(target: MAIN_DEBUG_GENERAL, "pills have been handed over");
}
