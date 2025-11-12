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
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::thread;
use tracing::trace;

use crate::options::{LaunchType, Options};
use crate::process::inferior::InferiorProcessReader;

mod inferior;

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
#[allow(dead_code)]
pub struct InferiorInner {
    /// PID of the inferior process.
    pub pid: Pid,
    /// PTY master fd (for resize/ioctl).
    pub master_fd: RawFd,
    /// Writer to stdin (own fd).
    pub writer: File,
    /// The raw file descriptor for the inferior's stdout/stderr.
    pub reader_fd: OwnedFd,
}

// TODO: this wrapper around the inner might turn out to be unnessary
pub struct Inferior {
    inner: InferiorInner,
}

pub enum TargetProcess {
    Inferior(Inferior),
    Attached(Pid),
    Disconnected,
}

impl TargetProcess {
    fn shutdown(&mut self) {
        match self {
            TargetProcess::Inferior(_inferior) => {
            },
            TargetProcess::Attached(_)
            | TargetProcess::Disconnected => {},
        }
    }
}

/// The primary struct containing information about the process being debugged.
#[allow(dead_code)]
pub struct Process {
    cli_options: Options,
    /// State of the inferior process.
    state: ProcessState,
    target_process: TargetProcess,
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
    shutdown_rx: Receiver<()>
}

impl Process {
    pub fn new(cli_options: Options, inferior_tx: Sender<String>, shutdown_rx: Receiver<()>) -> Process {
        // Note: this is slightly borked for PID-based launches :shrug:
        let opts = match cli_options.launch_type {
            LaunchType::Pid { pid: _ } => cli_options,
            LaunchType::Name { name } => Options {
                launch_type: LaunchType::Name { name },
                history_file: cli_options.history_file.clone(),
            },
        };

        Process {
            cli_options: opts,
            state: ProcessState::Unknown,
            target_process: TargetProcess::Disconnected,
            inferior_output: Vec::new(),
            inferior_tx,
            shutdown_rx,
        }
    }

    /// Attach to the process, spawning a new process if we only have a name.
    pub fn attach(&mut self, args: Vec<String>) -> Result<()> {
        match self.cli_options.launch_type {
            LaunchType::Pid { pid } => {
                trace!("Attaching to pid {:?}", pid);
                let pid = attach_pid(pid)?;
                self.target_process = TargetProcess::Attached(pid);
            }
            LaunchType::Name { ref name } => {
                trace!("Spawning inferior process {:?}", name);
                self.inferior_output.clear();
                let inferior_inner =
                    launch_file(name, args)?.expect("Should receive inferior process info");

                // start inferior reader
                let mut reader = InferiorProcessReader {
                    fd: inferior_inner.reader_fd.try_clone()?,
                    send_channel: self.inferior_tx.clone(),
                    shutdown_channel: self.shutdown_rx.clone(),
                };
                thread::spawn(move || {
                    reader.run();
                });

                let inferior = Inferior {
                    inner: inferior_inner,
                };
                self.target_process = TargetProcess::Inferior(inferior);
            }
        }

        // TODO: not sure about setting the state here to Running ...
        self.state = ProcessState::Running;
        self.wait_on_signal()?;
        Ok(())
    }

    fn pid(&self) -> Option<Pid> {
        match self.target_process {
            TargetProcess::Inferior(ref inferior) => Some(inferior.inner.pid),
            TargetProcess::Attached(pid) => Some(pid),
            TargetProcess::Disconnected => None,
        }
    }

    fn expect_pid(&self) -> Pid {
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
                self.target_process.shutdown();
            }
            WaitStatus::Signaled(_, _, _) => {
                self.state = ProcessState::Terminated;
                self.target_process.shutdown();
            }
            WaitStatus::Stopped(_, _) => self.state = ProcessState::Stopped,
            _ => {},
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

        // if the debugger launched the process, we need to kill it
        if self.cli_options.launch_type.terminate_on_exit() {
            kill(pid, Some(Signal::SIGKILL))?;
            self.wait_on_signal()?;
        } else {
            self.state = ProcessState::Unknown;
        }

        self.target_process.shutdown();
        
        Ok(())
    }

    pub fn receive_inferior_logging(&mut self, output: String) {
        self.inferior_output.push(output);
    }

    pub fn last_n_log_lines(&self, n: usize) -> &[String] {
        let len = self.inferior_output.len().saturating_sub(n);
        &self.inferior_output[len..]
    }
}

fn attach_pid(pid: i32) -> Result<Pid> {
    // PID should have been checked earlier (so that's it's a legit value, > 0)
    let p = Pid::from_raw(pid);
    ptrace::attach(p)?;
    Ok(p)
}

fn launch_file(name: &Path, _args: Vec<String>) -> Result<Option<InferiorInner>> {
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

            Ok(Some(InferiorInner {
                pid: child,
                master_fd: pty.master.as_raw_fd(),
                reader_fd: rfd.try_clone()?,
                writer,
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
            let cstr_args: Vec<&CStr> = vec![];

            let _ = execvp(filename.as_c_str(), &cstr_args);
            Ok(None)
        }
    }
}
