use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use nameof::name_of;
use tracing::{instrument, trace};

use crate::program::program_messages::Message;
use crate::program::ProgramData;

#[derive(Copy, Clone, Debug)]
pub struct EngineData {}

#[instrument(ret)]
pub(crate) fn engine_thread(thread_start_barrier: Arc<Barrier>, program_data_wrapped: Arc<Mutex<ProgramData>>, program_message_sender: flume::Sender<Message>) -> color_eyre::eyre::Result<()> {
    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running engine thread");

    loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}