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
    let old_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        old_hook(panic_info);
        error!(
            target: REALLY_FUCKING_BAD_UNREACHABLE,
            "you chose the red pill, please stand by while process is ejected from OS matrix"
        );
        error!(
            target: REALLY_FUCKING_BAD_UNREACHABLE,
            ?panic_info,
            "process panicked. process will now exit\n{panic_info}"
        );
        std::process::abort();
    }));
    debug!(target: MAIN_DEBUG_GENERAL, "pills have been handed over");
}
