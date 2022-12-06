use std::{process, thread};

use tracing::error;

/// struct that if dropped when the thread is panicking, prints a log message and quits the whole process
pub struct NoPanicPill;

impl Drop for NoPanicPill {
    fn drop(&mut self) {
        if thread::panicking() {
            error!("dropped while unwinding. process will now exit");
            process::exit(-1);
        }
    }
}