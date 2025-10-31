pub mod options;

use anyhow::{anyhow, Result};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{ForkResult, Pid, execv, fork};
use rustyline::config::BellStyle;
use rustyline::{Config, DefaultEditor};
use std::path::Path;

use crate::options::{LaunchType, Options};

const HISTORY_FILE: &str = "~/.cache/jdb/history";

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
    /// Flag if the program is currently being debugged
    debugging: bool,
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
                debugging: false,
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
                debugging: false,
                state: ProcessState::Stopped,
                arguments: Some(args),
            },
        }
    }

    pub fn attach(&mut self) -> Result<()> {
        if self.debugging {
            return Ok(());
        }

        let pid = match self.cli_options.launch_type {
            LaunchType::Pid { pid } => attach_pid(pid)?,
            LaunchType::Name { ref name, .. } => attach_path(name, &self.arguments)?,
        };
        self.debugging = true;
        self.pid = Some(pid);
        Ok(())
    }

    pub fn resume(&self) -> Result<()> {
        if !self.debugging || self.pid.is_none() {
            return Err(anyhow!("process is not being debugged currently"));
        }

        let pid = self.pid.unwrap();
        ptrace::cont(pid, None)?;
        Ok(())
    }
}

fn attach_pid(pid: i32) -> Result<Pid> {
    // TODO: check that pid > 0
    let p = Pid::from_raw(pid);
    ptrace::attach(p)?;
    Ok(p)
}

fn attach_path(name: &Path, args: &Option<String>) -> Result<Pid> {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None)?;
            Ok(child)
        }
        Ok(ForkResult::Child) => {
            // set the child as tracable
            ptrace::traceme()?;

            let cstr_args = if let Some(a) = args {
                vec![a]
            } else {
                vec![]
            };
            
            // TODO: impl me!!
            // now spawn the program
            // execv(name, &cstr_args)?;

            // return _some_ PID-looking thing ...
            Ok(Pid::from_raw(0))
        }
        Err(e) => Err(anyhow!(e)),
    }
}

pub struct Debugger {
    line_reader: DefaultEditor,
}

impl Debugger {
    pub fn new() -> Result<Debugger> {
        let config = Config::builder()
            .edit_mode(rustyline::EditMode::Emacs)
            .max_history_size(10000)?
            .bell_style(BellStyle::None)
            .tab_stop(4)
            .build();
        let mut line_reader = DefaultEditor::with_config(config)?;
        let _ = line_reader.load_history(HISTORY_FILE);

        Ok(Debugger {
            line_reader,
        })
    }

    pub fn next(&mut self, process: &mut Process) -> Result<()> {
        let line = self.line_reader.readline("(jdb) ")?;
        println!("{:?}", line);

        // history mgmt
        let _ = self.line_reader.add_history_entry(line.as_str());
        self.line_reader.append_history(HISTORY_FILE)?;

        // assume 'continue' and wait for the inferior process
        process.resume()?;
        waitpid(process.pid, None)?;    

        Ok(())
    }
}
