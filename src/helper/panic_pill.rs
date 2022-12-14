use std::panic::PanicInfo;

use tracing::{debug, error};

use crate::helper::logging::event_targets::REALLY_FUCKING_BAD_UNREACHABLE;
use crate::helper::logging::event_targets::MAIN_DEBUG_GENERAL;

/// Got panics? We've got a pill for that!
///
/// Custom panic hook that prints a log message and quits the whole process
fn swallow_panic_pill<F>(old_hook: F, panic_info: &PanicInfo<'_>)
                             where F: Fn(&PanicInfo<'_>) + Send + Sync + 'static
{
    old_hook(panic_info);
    error!("you chose the red pill, please stand by while process is be ejected from OS matrix");
    error!(target: REALLY_FUCKING_BAD_UNREACHABLE,"process panicked. process will now exit");
    std::process::abort();
}

pub(crate) fn red_or_blue_pill() {
    debug!(target: MAIN_DEBUG_GENERAL, "generating a red an blue pill, choose wisely");
    std::panic::update_hook(Box::new(swallow_panic_pill));
    debug!(target: MAIN_DEBUG_GENERAL, "pills have been handed over");
}