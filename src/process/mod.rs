use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use nix::libc;
use nix::pty::{Winsize, openpty};
use nix::sys::ptrace;
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{
    ForkResult, Pid, close, dup, dup2_stderr, dup2_stdin, dup2_stdout, execvp, fork, setsid,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::thread::{self, JoinHandle};
use tracing::trace;

use crate::debugger::BreakpointCommand;
use crate::options::Options;
use crate::process::inferior::read_inferior_logging;
use crate::process::register_info::{Register, RegisterValue};
use crate::process::registers::{RegisterSnapshot, read_all_registers};
use crate::process::stoppoint::breakpoint_site::BreakpointSite;
use crate::process::stoppoint::{INTERRUPT_INSTRUCTION, StoppointId, VirtualAddress};

mod inferior;
pub mod register_info;
mod registers;
pub mod stoppoint;

#[derive(Clone, Debug)]
pub enum ProcessState {
    /// Debugger hasn't attached to or launched the inferior process, so we don't
    /// know what it's state is yet.
    Unknown,
    /// The inferior process is stopped, awaiting a nudge from debugger.
    Stopped,
    Running,
    /// The inferior process exited normally.
    Exited,
    /// The inferior process terminated, either normally or forcefully.
    Terminated,
}

/// Represents a process ("inferior") that the debugger has spawned
/// under a pseudo-terminal (PTY).  
///
/// This structure owns all handles necessary for I/O, resizing, and
/// signal control of the inferior process.  It is the debugger’s view
/// of “the program being debugged.”
#[derive(Debug)]
pub struct Inferior {
    /// PID of the inferior process.
    pid: Pid,
    /// PTY master fd (for resize/ioctl).
    pub master_fd: RawFd,
    /// Writer to stdin (own fd).
    pub writer: File,
    /// The raw file descriptor for the inferior's stdout/stderr.
    pub reader_fd: OwnedFd,

    /// The active, enablkes breakpoints on this running inferior.
    /// The map's values are the original instructions that we replaced with
    /// `int3`.
    breakpoint_sites: HashMap<StoppointId, u8>,
}

impl Inferior {
    pub fn pid(&self) -> Pid {
        self.pid
    }

    fn enable_breakpoint_site(&mut self, breakpoint_site: &BreakpointSite) -> Result<()> {
        if self.breakpoint_sites.contains_key(&breakpoint_site.id()) {
            // not sure if we should error or just silently return
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

    fn disable_breakpoint_site(&mut self, breakpoint_site: &BreakpointSite) -> Result<()> {
        let saved_instruction = match self.breakpoint_sites.remove(&breakpoint_site.id()) {
            Some(v) => v,
            None => {
                return Ok(());
            }
        };

        if !self.breakpoint_sites.contains_key(&breakpoint_site.id()) {
            // not sure if we should error or just silently return
            return Ok(());
        }

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

/// The primary struct containing information about the process being debugged.
#[allow(dead_code)]
pub struct Process {
    cli_options: Options,
    /// State of the inferior process.
    state: ProcessState,
    target_process: Option<Inferior>,
    registers: Option<RegisterSnapshot>,
    /// Captured stdout/stderr from the inferior process.
    /// We reason the inferior output is stored here, rather than in
    /// `Inferior` is that we'd like the output to still be available
    /// for tui rendering even after the inferior has existed (and we've
    /// tansistioned the state/target_process).
    /// -- I might revisit this decision, though.
    // Vec is a starting point/placeholder for now, would prefer
    // something like a circular buffer
    inferior_output: Vec<String>,
    inferior_tx: Sender<String>,
    shutdown_rx: Receiver<()>,
    logging_thread: Option<JoinHandle<()>>,

    breakpoint_sites: Vec<BreakpointSite>,
}

impl Process {
    pub fn new(
        cli_options: Options,
        inferior_tx: Sender<String>,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        // Note: this is slightly borked for PID-based launches :shrug:
        Process {
            cli_options,
            state: ProcessState::Unknown,
            target_process: None,
            inferior_output: Vec::new(),
            registers: None,
            inferior_tx,
            shutdown_rx,
            logging_thread: None,
            breakpoint_sites: Default::default(),
        }
    }

    /// Attach to the process by spawning a new process for the configured executable.
    pub fn attach(&mut self, args: Vec<String>) -> Result<()> {
        trace!(
            "Spawning inferior process {:?}",
            self.cli_options.executable
        );
        self.inferior_output.clear();
        let inferior = launch_executable(self.cli_options.executable.as_path(), args)?
            .expect("Should receive inferior process info");

        let fd_clone = inferior.reader_fd.try_clone()?;
        let inferior_tx_clone = self.inferior_tx.clone();
        let shutdown_rx_clone = self.shutdown_rx.clone();

        // start inferior reader
        let logging_thread = thread::spawn(move || {
            read_inferior_logging(fd_clone, inferior_tx_clone, shutdown_rx_clone);
        });
        self.logging_thread = Some(logging_thread);
        self.target_process = Some(inferior);

        // TODO: not sure about setting the state here to Running ...
        self.state = ProcessState::Running;
        self.wait_on_signal()?;

        // now that the inferior is ready, set any enabled breakpoints.
        // TODO: check WaitStatus is good before trying to set the breakpoints.
        let inferior = self.target_process.as_mut().expect("just created");
        for b in self.breakpoint_sites.iter() {
            if b.is_enabled() {
                inferior.enable_breakpoint_site(b)?;
            }
        }

        Ok(())
    }

    pub fn pid(&self) -> Option<Pid> {
        if let Some(ref inferior) = self.target_process {
            return Some(inferior.pid());
        }
        None
    }

    pub fn expect_pid(&self) -> Pid {
        self.pid().expect("Should have PID at this point")
    }

    pub fn resume(&mut self) -> Result<()> {
        if !matches!(self.state, ProcessState::Stopped | ProcessState::Running) {
            return Err(anyhow!("Inferior process not being debugged"));
        }

        let pid = self.expect_pid();
        ptrace::cont(pid, None)?;
        self.state = ProcessState::Running;

        Ok(())
    }

    pub fn wait_on_signal(&mut self) -> Result<WaitStatus> {
        let wait_status = waitpid(self.expect_pid(), None)?;

        // if exited/terminated, send shutdown signal to inferior reader
        match wait_status {
            WaitStatus::Exited(_, _) => {
                self.state = ProcessState::Exited;
            }
            WaitStatus::Signaled(_, _, _) => {
                self.state = ProcessState::Terminated;
            }
            WaitStatus::Stopped(_, _) => self.state = ProcessState::Stopped,
            _ => {}
        };

        if matches!(self.state, ProcessState::Stopped) {
            self.registers = Some(read_all_registers(self.expect_pid())?);
        }

        Ok(wait_status)
    }

    pub fn destroy(&mut self) -> Result<()> {
        if !matches!(self.state, ProcessState::Running) {
            return Ok(());
        }

        let pid = self.expect_pid();

        // tell the inferior to STOP and wait for it
        kill(pid, Some(Signal::SIGSTOP))?;
        waitpid(pid, None)?;

        // let the inferior know we are done tracing it
        ptrace::detach(pid, None)?;
        kill(pid, Some(Signal::SIGCONT))?;

        // we launched the inferior process, so we should reap it here
        kill(pid, Some(Signal::SIGKILL))?;
        self.wait_on_signal()?;

        if let Some(handle) = self.logging_thread.take() {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn receive_inferior_logging(&mut self, output: String) {
        output.lines().for_each(|l| {
            if !l.is_empty() {
                self.inferior_output.push(l.to_string());
            }
        });
    }

    pub fn last_n_log_lines(&self, n: usize) -> &[String] {
        let len = self.inferior_output.len().saturating_sub(n);
        &self.inferior_output[len..]
    }

    pub fn read_register(&self, register: Register) -> Option<RegisterValue> {
        // TODO: maybe add check to ensure target process is indeed running/being debugged,
        // but perhaps having self.registers may be sufficient

        self.registers
            .as_ref()
            .map(|snapshot| snapshot.read(&register))
    }

    pub fn breakpoint_command(&mut self, command: BreakpointCommand) -> Result<()> {
        // TODO: rewrite this function, and maybe change the Vec -> HashMap ??
        match command {
            BreakpointCommand::Create(address) => {
                let b = self.create_breakpoint_site(address)?;
                if let Some(inferior) = self.target_process.as_mut() {
                    inferior.enable_breakpoint_site(&b)?;
                }
            }
            BreakpointCommand::Delete(id) => {
                // the mutliple iterations are kinda weak ...
                let b = match self.breakpoint_sites.iter().find(|b| b.id() == id) {
                    Some(b) => b,
                    None => {
                        return Err(anyhow!("Cannot find breakpoitn by id {:?}", id));
                    }
                };

                if let Some(inferior) = self.target_process.as_mut() {
                    inferior.disable_breakpoint_site(b)?
                }

                self.breakpoint_sites.retain(|b| b.id() != id);
            }
            BreakpointCommand::Enable(id) => {
                for b in self.breakpoint_sites.iter_mut() {
                    if b.id() == id {
                        if let Some(inferior) = self.target_process.as_mut() {
                            inferior.enable_breakpoint_site(b)?;
                        }
                        b.enable();
                    }
                }
            }
            BreakpointCommand::Disable(id) => {
                for b in self.breakpoint_sites.iter_mut() {
                    if b.id() == id {
                        if let Some(inferior) = self.target_process.as_mut() {
                            inferior.disable_breakpoint_site(b)?;
                        }
                        b.disable();
                    }
                }
            }
        }
        Ok(())
    }

    fn create_breakpoint_site(&mut self, address: VirtualAddress) -> Result<BreakpointSite> {
        if self.breakpoint_sites.iter().any(|b| b.address() == address) {
            // either silently ignore (and return existing value) or return error?
            return Err(anyhow!(
                "Breakpoint site already exists for address {:?}",
                address
            ));
        }

        let b = BreakpointSite::new(address);
        self.breakpoint_sites.push(b.clone());
        Ok(b)
    }
}

fn launch_executable(name: &Path, args: Vec<String>) -> Result<Option<Inferior>> {
    let pty = openpty(
        Some(&Winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }),
        None,
    )?;
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            // Parent keeps master; close slave
            let _ = close(pty.slave);

            // Duplicate master for independent reader/writer File ownership
            let rfd = dup(pty.master.try_clone()?)?;
            let wfd = dup(pty.master.try_clone()?)?;

            let writer = File::from(wfd);

            Ok(Some(Inferior {
                pid: child,
                master_fd: pty.master.as_raw_fd(),
                reader_fd: rfd.try_clone()?,
                writer,
                breakpoint_sites: Default::default(),
            }))
        }
        ForkResult::Child => {
            setsid()?;
            // make slave controlling TTY
            unsafe { libc::ioctl(pty.slave.as_raw_fd(), libc::TIOCSCTTY, 0) };

            dup2_stdin(pty.slave.try_clone()?)?;
            dup2_stdout(pty.slave.try_clone()?)?;
            dup2_stderr(pty.slave.try_clone()?)?;
            let _ = close(pty.slave.try_clone()?);
            let _ = close(pty.master);

            ptrace::traceme()?;

            let filename = CString::new(name.as_os_str().as_bytes())?;

            // Build argv as &[&CStr] while retaining owned CString storage.
            let mut cstr_storage = Vec::with_capacity(args.len() + 1);
            cstr_storage.push(filename.clone());
            for arg in args {
                cstr_storage.push(CString::new(arg)?);
            }
            let cstr_args: Vec<&CStr> = cstr_storage.iter().map(|s| s.as_c_str()).collect();

            let _ = execvp(filename.as_c_str(), &cstr_args);
            Ok(None)
        }
    }
}
