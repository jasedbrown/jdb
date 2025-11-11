use anyhow::{Result, anyhow};
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
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use tracing::trace;

use crate::options::{LaunchType, Options};

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
pub struct Inferior {
    /// PID of the inferior process.
    pub pid: Pid,
    /// PTY master fd (for resize/ioctl).
    pub master_fd: RawFd,
    /// Reader for merged stdout/stderr (own fd).
    pub reader: File,
    /// Writer to stdin (own fd).
    pub writer: File,
}

pub enum TargetProcess {
    Inferior(Inferior),
    Attached(Pid),
    Disconnected,
}

/// The primary struct containing information about the process being debugged.
#[allow(dead_code)]
pub struct Process {
    cli_options: Options,
    /// State of the inferior process.
    state: ProcessState,

    /// `Option` if we have connect to an existing process yet, or have not spawned
    /// an inferior process yet.
    target_process: TargetProcess,
}

impl Process {
    pub fn new(cli_options: Options) -> Process {
        match cli_options.launch_type {
            LaunchType::Pid { pid: _ } => Process {
                cli_options,
                state: ProcessState::Unknown,
                target_process: TargetProcess::Disconnected,
            },
            LaunchType::Name { name } => Process {
                cli_options: Options {
                    launch_type: LaunchType::Name { name },
                    history_file: cli_options.history_file.clone(),
                },
                state: ProcessState::Unknown,
                target_process: TargetProcess::Disconnected,
            },
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
                let inferior =
                    launch_file(name, args)?.expect("Should receive inferior process info");
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
            TargetProcess::Inferior(ref inferior) => Some(inferior.pid),
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

        match wait_status {
            WaitStatus::Exited(_, _) => self.state = ProcessState::Exited,
            WaitStatus::Signaled(_, _, _) => self.state = ProcessState::Terminated,
            WaitStatus::Stopped(_, _) => self.state = ProcessState::Stopped,
            _ => (),
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

        // if the debugger launched the process, we need to kill it
        if self.cli_options.launch_type.terminate_on_exit() {
            kill(pid, Some(Signal::SIGKILL))?;
            self.wait_on_signal()?;
        } else {
            self.state = ProcessState::Unknown;
        }

        Ok(())
    }
}

fn attach_pid(pid: i32) -> Result<Pid> {
    // PID should have been checked earlier (so that's it's a legit value, > 0)
    let p = Pid::from_raw(pid);
    ptrace::attach(p)?;
    Ok(p)
}

fn launch_file(name: &Path, _args: Vec<String>) -> Result<Option<Inferior>> {
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
            let rfd = dup(pty.master.try_clone()?)?; // reader fd
            let wfd = dup(pty.master.try_clone()?)?; // writer fd

            let reader = File::from(rfd);
            let writer = File::from(wfd);

            Ok(Some(Inferior {
                pid: child,
                master_fd: pty.master.as_raw_fd(),
                reader,
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
