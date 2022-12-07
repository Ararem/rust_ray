use std::{process, thread};

use tracing::{debug, debug_span, error};

/// struct that if dropped when the thread is panicking, prints a log message and quits the whole process
pub struct PanicPill;

impl Drop for PanicPill {
    fn drop(&mut self) {
        let _span = debug_span!("panic_pill_drop").entered();
        if thread::panicking() {
            error!("pill dropped while unwinding. process will now exit");
            process::exit(-1);
            // process::abort();
        } else {
            debug!("pill dropped normally")
        }
        _span.exit();
    }
}