use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::io::Read;
use std::os::fd::{AsRawFd, OwnedFd};
use std::time::Duration;
use tracing::{error, trace};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use std::os::unix::io::FromRawFd;

/// It's actually the PTY's merged stdout/stderr
const STDOUT: Token = Token(0);

pub fn read_inferior_logging(
    fd: OwnedFd,
    send_channel: Sender<String>,
    shutdown_channel: Receiver<()>,
) {
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    let mut source_fd = SourceFd(&fd.as_raw_fd());

    poll.registry()
        .register(&mut source_fd, STDOUT, Interest::READABLE)
        .unwrap();

    let mut file = unsafe { std::fs::File::from_raw_fd(fd.as_raw_fd()) };
    let mut buffer = [0u8; 4096];

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(42)))
            .unwrap();
        for event in events.iter() {
            if event.token() != STDOUT {
                trace!(
                    ?event,
                    "Received notification about a type we don't process"
                );
                continue;
            }
            if event.is_readable() {
                match file.read(&mut buffer) {
                    Ok(0) => {
                        trace!("EOF reached");
                        // TODO: WTF???
                        return;
                    }
                    Ok(n) => {
                        // TODO: process buffer ... but how it converts for UTF-8 for now ...
                        let s = String::from_utf8_lossy(&buffer[..n]);
                        if let Err(e) = send_channel.send(s.into_owned()) {
                            error!("Error when sending to loggin_tx channel: {:?}", e)
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // nop, ignore
                    }
                    Err(e) => {
                        error!(?e, "Error while reading inferior process out");
                    }
                }
            }

            match shutdown_channel.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    trace!("Stop signal received at inferior reader");
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}
