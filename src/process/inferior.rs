use std::os::fd::OwnedFd;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use tracing::{error, trace};

/// A struct to live in a daemon thread to read the stdout/stderr of the inferior process
#[allow(dead_code)]
pub struct InferiorProcessReader {
    /// File descriptor of the stdout/stderr channel.
    pub fd: OwnedFd,

    /// A channel to publish any data read from the inferior's stdout/stderr.
    pub send_channel: Sender<String>,

    /// A simple shutdown channel.
    pub shutdown_channel: Receiver<()>,
}

impl InferiorProcessReader {
    pub fn run(&mut self) {
        loop {
            match self.shutdown_channel.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    trace!("Stop signal received at inferior reader");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
            
            std::thread::sleep(Duration::from_millis(10));

            // silly send
            if let Err(e) = self.send_channel.send("hello, jdb".to_string()) {
                error!("Error when sending to loggin_tx channel: {:?}", e)
            }
        }
    }
}
