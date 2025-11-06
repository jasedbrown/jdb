use anyhow::{Result, anyhow};
use nix::sys::ptrace;
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, Pid, execvp, fork};
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

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

#[derive(Clone, Debug)]
/// The primary struct containing information about the process being debugged.
pub struct Process {
    cli_options: Options,
    /// State of the inferior process.
    state: ProcessState,
    /// PID of the process. Optional in case it's not currently running (and need to be
    /// spawned as an inferior process).
    pid: Option<Pid>,
}

impl Process {
    pub fn new(cli_options: Options) -> Process {
        match cli_options.launch_type {
            LaunchType::Pid { pid } => Process {
                cli_options,
                pid: Some(Pid::from_raw(pid)),
                state: ProcessState::Unknown,
            },
            LaunchType::Name { name } => Process {
                cli_options: Options {
                    launch_type: LaunchType::Name { name },
                    history_file: cli_options.history_file.clone(),
                },
                pid: None,
                state: ProcessState::Unknown,
            },
        }
    }

    /// Attach to the process, spawning a new process if we only have a name.
    pub fn attach(&mut self, args: Vec<String>) -> Result<()> {
        let pid = match self.cli_options.launch_type {
            LaunchType::Pid { pid } => attach_pid(pid)?,
            LaunchType::Name { ref name } => launch_file(name, args)?,
        };

        // TODO: not sure about setting the state here to Running ...
        self.state = ProcessState::Running;
        self.pid = Some(pid);
        self.wait_on_signal()?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        if !matches!(self.state, ProcessState::Stopped | ProcessState::Running) {
            return Err(anyhow!("Inferior process not being debugged"));
        }

        let pid = self.pid.unwrap();
        ptrace::cont(pid, None)?;
        self.state = ProcessState::Running;

        Ok(())
    }

    pub fn wait_on_signal(&mut self) -> Result<WaitStatus> {
        let wait_status = waitpid(self.pid, None)?;

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

        let pid = self.pid.expect("PID should be a value");

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

fn launch_file(name: &Path, _args: Vec<String>) -> Result<Pid> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => Ok(child),
        ForkResult::Child => {
            // set the child as tracable
            ptrace::traceme()?;

            let filename = CString::new(name.as_os_str().as_bytes())?;
            let cstr_args: Vec<&CStr> = vec![];

            let _ = execvp(filename.as_c_str(), &cstr_args);

            // "return" some PID-looking thing to make the compiler happy
            Ok(Pid::from_raw(0))
        }
    }
}
