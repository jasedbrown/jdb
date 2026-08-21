use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use nix::libc;
use nix::pty::{Winsize, openpty};
use nix::sys::personality::{self, Persona};
use nix::sys::ptrace;
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{
    ForkResult, Pid, close, dup, dup2_stderr, dup2_stdin, dup2_stdout, execvp, fork, setsid,
};

use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::thread::{self, JoinHandle};
use tracing::trace;

use crate::debugger::BreakpointCommand;
use crate::options::{Aslr, Options};
use crate::process::inferior::{Inferior, read_inferior_logging};
use crate::process::register_info::{Register, RegisterValue};
use crate::process::registers::{RegisterSnapshot, read_all_registers};
use crate::process::stoppoint::VirtualAddress;
use crate::process::stoppoint::breakpoint_site::BreakpointSite;

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
    /// The inferior process is, unsurprisingly, running.
    Running,
    /// The inferior process exited normally.
    Exited,
    /// The inferior process terminated, either normally or forcefully.
    Terminated,
}

/// The primary struct containing information about the process being debugged.
#[allow(dead_code)]
pub struct Process {
    cli_options: Options,
    /// State of an inferior process.
    state: ProcessState,
    /// The inferior being debugged. Will be `None` if the process has not executed
    /// or has exited.
    inferior_process: Option<Inferior>,
    /// Snapshot of the inferior's register values. Maintained independently
    /// of the `inferior_process` such that the regisrters can be inspected
    /// after the inferior exits.
    registers: Option<RegisterSnapshot>,
    /// Captured stdout/stderr from the inferior process.
    ///
    /// The reason the inferior output is stored here, rather than in
    /// `Inferior` is that we'd like the output to still be available
    /// for tui rendering even after the inferior has existed (and we've
    /// transistioned the state/inferior_process).
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
            inferior_process: None,
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
        let inferior =
            launch_executable(self.cli_options.executable.as_path(), args, Aslr::Enabled)?
                .expect("Should receive inferior process info");

        let fd_clone = inferior.reader_fd.try_clone()?;
        let inferior_tx_clone = self.inferior_tx.clone();
        let shutdown_rx_clone = self.shutdown_rx.clone();

        // start inferior reader
        let logging_thread = thread::spawn(move || {
            read_inferior_logging(fd_clone, inferior_tx_clone, shutdown_rx_clone);
        });
        self.logging_thread = Some(logging_thread);
        self.inferior_process = Some(inferior);

        // TODO: not sure about setting the state here to Running ...
        self.state = ProcessState::Running;
        self.wait_on_signal()?;

        // now that the inferior is ready, set any enabled breakpoints.
        // TODO: check WaitStatus is good before trying to set the breakpoints.
        let inferior = self.inferior_process.as_mut().expect("just created");
        for b in self.breakpoint_sites.iter() {
            if b.is_enabled() {
                inferior.enable_breakpoint_site(b)?;
            }
        }

        Ok(())
    }

    pub fn pid(&self) -> Option<Pid> {
        if let Some(ref inferior) = self.inferior_process {
            return Some(inferior.pid());
        }
        None
    }

    pub fn expect_pid(&self) -> Pid {
        self.pid().expect("Should have PID at this point")
    }

    /// Continue (resume) debugging the inferior process.
    ///
    /// Essentially does `PTRACE_CONT`.
    pub fn resume(&mut self) -> Result<()> {
        if !matches!(self.state, ProcessState::Stopped | ProcessState::Running) {
            return Err(anyhow!("Inferior process not being debugged"));
        }

        let pid = self.expect_pid();
        ptrace::cont(pid, None)?;
        self.state = ProcessState::Running;

        Ok(())
    }

    pub fn set_pc(&mut self, address: VirtualAddress) -> Result<()> {
        let Some(registers) = self.registers.as_mut() else {
            return Err(anyhow!("No registers yet"));
        };
        registers.set_pc(address)
    }

    /// Wait for the inferior to change it's status (i.e. hit a breakpoint
    /// or exit/terminate).
    pub fn wait_on_signal(&mut self) -> Result<WaitStatus> {
        let wait_status = waitpid(self.expect_pid(), None)?;
        trace!("signal received: {:?}", &wait_status);

        // TODO: if exited/terminated, send shutdown signal to inferior reader
        match wait_status {
            WaitStatus::Exited(_, _) => {
                self.state = ProcessState::Exited;
            }
            WaitStatus::Signaled(_, _, _) => {
                self.state = ProcessState::Terminated;
            }
            WaitStatus::Stopped(_, signal) => {
                let mut registers = read_all_registers(self.expect_pid())?;
                if matches!(signal, Signal::SIGTRAP) {
                    // set the PC back one, to where the breakpoint currently is
                    let cur_pc = registers.get_pc()?;
                    let instr_begin = VirtualAddress::from(cur_pc.address - 1_u64);
                    if self
                        .breakpoint_sites
                        .iter()
                        .any(|b| b.address() == instr_begin && b.is_enabled())
                    {
                        registers.set_pc(instr_begin)?;
                    }
                }
                self.registers = Some(registers);
                self.state = ProcessState::Stopped
            }
            _ => {}
        };

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

    /// React to a breakpoint command the user has issued.
    pub fn breakpoint_command(&mut self, command: BreakpointCommand) -> Result<()> {
        // TODO: rewrite this function, and maybe change the Vec -> HashMap ??
        match command {
            BreakpointCommand::Create(address) => {
                let b = self.create_breakpoint_site(address)?;
                if let Some(inferior) = self.inferior_process.as_mut() {
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

                if let Some(inferior) = self.inferior_process.as_mut() {
                    inferior.disable_breakpoint_site(b)?
                }

                // the second full iteration is weak, but largely inconsequential perf-wise.
                self.breakpoint_sites.retain(|b| b.id() != id);
            }
            BreakpointCommand::Enable(id) => {
                for b in self.breakpoint_sites.iter_mut() {
                    if b.id() == id {
                        if let Some(inferior) = self.inferior_process.as_mut() {
                            inferior.enable_breakpoint_site(b)?;
                        }
                        b.enable();
                    }
                }
            }
            BreakpointCommand::Disable(id) => {
                for b in self.breakpoint_sites.iter_mut() {
                    if b.id() == id {
                        if let Some(inferior) = self.inferior_process.as_mut() {
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

fn launch_executable(
    name: &Path,
    inferior_args: Vec<String>,
    aslr: Aslr,
) -> Result<Option<Inferior>> {
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
                _master_fd: pty.master.as_raw_fd(),
                reader_fd: rfd.try_clone()?,
                _writer: writer,
                breakpoint_sites: Default::default(),
            }))
        }
        ForkResult::Child => {
            // disable address space randomization (ASLR)
            if matches!(aslr, Aslr::Disabled) {
                personality::set(Persona::ADDR_NO_RANDOMIZE)?;
            }

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
            let mut cstr_storage = Vec::with_capacity(inferior_args.len() + 1);
            cstr_storage.push(filename.clone());
            for arg in inferior_args {
                cstr_storage.push(CString::new(arg)?);
            }
            let cstr_args: Vec<&CStr> = cstr_storage.iter().map(|s| s.as_c_str()).collect();

            let _ = execvp(filename.as_c_str(), &cstr_args);
            Ok(None)
        }
    }
}
