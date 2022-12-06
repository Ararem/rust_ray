use std::io::stdin;
use std::sync::{Arc, Barrier, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use color_eyre::Report;
use nameof::name_of;
use tracing::{info, instrument, trace};

use crate::program::program_messages::{Message, QuitAppErrorReason, QuitAppNoErrorReason};
use crate::program::program_messages::ProgramThreadMessage::QuitAppNoError;
use crate::program::ProgramData;
use crate::ui;

#[derive(Copy, Clone, Debug)]
pub struct UiData {}

#[instrument(ret, skip_all)]
pub(crate) fn ui_thread(thread_start_barrier: Arc<Barrier>, program_data_wrapped: Arc<Mutex<ProgramData>>, program_message_sender: flume::Sender<Message>) -> color_eyre::eyre::Result<()> {
    trace!("waiting for {}", name_of!(thread_start_barrier));
    thread_start_barrier.wait();
    trace!("wait complete, running engine thread");

    loop {
        // Pretend we're doing work here
        thread::sleep(Duration::from_secs(1));

        let mut s = String::new();
        if let Ok(_) = stdin().read_line(&mut s) {
            info!("sending quit");
            program_message_sender.send(
                Message::Program(QuitAppNoError(QuitAppNoErrorReason::QuitInteractionByUser))
            );
        }
    }

    Ok(())
}