use anyhow::{Result, anyhow};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, Pid, execvp, fork};
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::options::{LaunchType, Options};

#[derive(Clone, Debug)]
pub enum ProcessState {
    Unknown,
    Stopped,
    Running,
    Exited,
    Terminated,
}

#[derive(Clone, Debug)]
/// The primary struct containing information about the process being debugged.
pub struct Process {
    cli_options: Options,
    /// State of the inferior process.
    state: ProcessState,
    /// Command line arguments to the process (if jdb has spawned it)
    arguments: Option<String>,
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
                arguments: None,
            },
            LaunchType::Name { name, args } => Process {
                cli_options: Options {
                    launch_type: LaunchType::Name {
                        name,
                        args: args.clone(),
                    },
                },
                pid: None,
                state: ProcessState::Stopped,
                arguments: Some(args),
            },
        }
    }

    /// Attach to the process, spawning a new process if we only have a name.
    pub fn attach(&mut self) -> Result<()> {
        // this is a good enough check for now, just don't hold it wrong
        if matches!(self.state, ProcessState::Running) {
            return Ok(());
        }

        let pid = match self.cli_options.launch_type {
            LaunchType::Pid { pid } => attach_pid(pid)?,
            LaunchType::Name { ref name, .. } => launch_file(name, &self.arguments)?,
        };

        waitpid(pid, None)?;

        self.state = ProcessState::Running;
        self.pid = Some(pid);
        Ok(())
    }

    pub fn resume(&self) -> Result<()> {
        if !matches!(self.state, ProcessState::Running) {
            return Err(anyhow!("process is not being debugged currently"));
        }

        let pid = self.pid.unwrap();
        ptrace::cont(pid, None)?;
        waitpid(pid, None)?;

        Ok(())
    }
}

fn attach_pid(pid: i32) -> Result<Pid> {
    // PID should have been checked earlier (so that's it's a legit value, > 0)
    let p = Pid::from_raw(pid);
    ptrace::attach(p)?;
    Ok(p)
}

fn launch_file(name: &Path, _args: &Option<String>) -> Result<Pid> {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None)?;
            Ok(child)
        }
        Ok(ForkResult::Child) => {
            // set the child as tracable
            ptrace::traceme()?;

            let filename = CString::new(name.as_os_str().as_bytes())?;
            let cstr_args: Vec<&CStr> = vec![];

            execvp(filename.as_c_str(), &cstr_args)?;

            // return _some_ PID-looking thing ...
            Ok(Pid::from_raw(0))
        }
        Err(e) => Err(anyhow!(e)),
    }
}
