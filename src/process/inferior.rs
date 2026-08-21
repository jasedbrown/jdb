use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use nix::unistd::Pid;
use std::io::Read;
use std::os::fd::RawFd;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::io::FromRawFd;
use std::time::Duration;
use tracing::{error, trace};

use nix::sys::ptrace;
use std::collections::HashMap;
use std::fs::File;

use crate::process::stoppoint::breakpoint_site::BreakpointSite;
use crate::process::stoppoint::{INTERRUPT_INSTRUCTION, StoppointId};

/// It's actually the PTY's merged stdout/stderr
const STDOUT: Token = Token(0);

/// Represents a process ("inferior") that the debugger has spawned
/// under a pseudo-terminal (PTY).  
///
/// This structure owns all handles necessary for I/O, resizing, and
/// signal control of the inferior process.  It is the debugger’s view
/// of “the program being debugged.”
#[derive(Debug)]
pub struct Inferior {
    /// PID of the inferior process.
    pub pid: Pid,
    /// PTY master fd (used for resize/ioctl).
    pub _master_fd: RawFd,
    /// Writer to inferior's stdin (own fd).
    pub _writer: File,
    /// The raw file descriptor for the inferior's stdout/stderr.
    pub reader_fd: OwnedFd,

    /// The active, enabled breakpoints on this running inferior.
    /// The map's values are the original instructions that we replaced with
    /// `int3`.
    pub breakpoint_sites: HashMap<StoppointId, u8>,
}

impl Inferior {
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Enable the breakpoint in the inferior process.
    pub fn enable_breakpoint_site(&mut self, breakpoint_site: &BreakpointSite) -> Result<()> {
        if self.breakpoint_sites.contains_key(&breakpoint_site.id()) {
            return Ok(());
        }

        let instruction_line = ptrace::read(self.pid, breakpoint_site.address().addr() as _)?;
        let saved_instruction = (instruction_line & 0xff) as u8;

        let new_instruction_line = (instruction_line & !0xFF) | INTERRUPT_INSTRUCTION;
        ptrace::write(
            self.pid,
            breakpoint_site.address().addr() as _,
            new_instruction_line,
        )?;

        self.breakpoint_sites
            .insert(breakpoint_site.id(), saved_instruction);

        Ok(())
    }

    /// Disable the breakpoint in the inferior process.
    pub fn disable_breakpoint_site(&mut self, breakpoint_site: &BreakpointSite) -> Result<()> {
        let saved_instruction = match self.breakpoint_sites.remove(&breakpoint_site.id()) {
            Some(v) => v,
            None => {
                return Ok(());
            }
        };

        let instruction_line = ptrace::read(self.pid, breakpoint_site.address().addr() as _)?;
        let restored_line = (instruction_line & !0xFF) | saved_instruction as i64;
        ptrace::write(
            self.pid,
            breakpoint_site.address().addr() as _,
            restored_line,
        )?;
        Ok(())
    }
}

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
