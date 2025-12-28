use anyhow::{Result, anyhow};
use tracing::trace;

use crate::history::CommandHistory;
use crate::process::Process;
use crate::process::stoppoint::{StoppointId, VirtualAddress};

pub struct Debugger {
    /// Flag if the program is currently being debugged.
    debugging: bool,
    /// A log of all the commands executed against the debugger, historical and current.
    history: CommandHistory,
}

impl Debugger {
    pub fn new() -> Result<Debugger> {
        let history = CommandHistory::new()?;
        Ok(Debugger {
            debugging: false,
            history,
        })
    }

    pub fn next(&mut self, command: String, process: &mut Process) -> Result<DispatchResult> {
        let mut command = command;
        if command.is_empty() {
            trace!("next editor command is empty line, will replay last command");
            match self.history.last_command() {
                Some(cmd) => command = cmd,
                None => return Ok(DispatchResult::Normal),
            }
        } else {
            self.history.add(&command)?;
        }

        let cmd = Command::try_from(command)?;
        let result = self.dispatch_command(cmd, process)?;

        Ok(result)
    }

    fn dispatch_command(
        &mut self,
        command: Command,
        process: &mut Process,
    ) -> Result<DispatchResult> {
        let mut res = DispatchResult::Normal;
        match command {
            Command::Run(args) => {
                process.attach(args)?;
                self.debugging = true;
            }
            Command::Continue => {
                process.resume()?;
                process.wait_on_signal()?;
            }
            Command::Breakpoint(cmd) => {
                process.breakpoint_command(cmd)?;
                // wait_on_signal?? i don't think so, but ....
            }
            Command::Quit => {
                process.destroy()?;
                self.debugging = false;
                res = DispatchResult::Exit;
            }
        }

        Ok(res)
    }

    pub fn is_debugging(&self) -> bool {
        self.debugging
    }
}

#[derive(Clone, Debug)]
pub enum DispatchResult {
    Normal,
    Exit,
}

#[derive(Clone, Debug)]
pub enum BreakpointCommand {
    Create(VirtualAddress),
    Delete(StoppointId),
    Enable(StoppointId),
    Disable(StoppointId),
}

#[derive(Clone, Debug)]
pub enum Command {
    /// Start or connect to the inferior process.
    Run(Vec<String>),
    Continue,
    Breakpoint(BreakpointCommand),
    /// Exit the debugger (and kill inferior process if it was launched).
    Quit,
}

impl TryFrom<String> for Command {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Command> {
        let mut words = value.split_whitespace();
        let cmd = words.next().unwrap_or("").to_string();
        let args: Vec<String> = words.map(|s| s.to_string()).collect();

        let command = match cmd.to_lowercase().as_str() {
            "run" | "r" => Command::Run(args),
            "continue" | "c" => Command::Continue,
            "quit" | "q" => Command::Quit,
            "break" | "b" => {
                Command::Breakpoint(BreakpointCommand::Create(VirtualAddress::try_from(args)?))
            }
            "delete" => {
                Command::Breakpoint(BreakpointCommand::Delete(StoppointId::try_from(args)?))
            }
            "enable" => {
                Command::Breakpoint(BreakpointCommand::Enable(StoppointId::try_from(args)?))
            }
            "disable" => {
                Command::Breakpoint(BreakpointCommand::Disable(StoppointId::try_from(args)?))
            }
            _ => return Err(anyhow!("unknown command: {:?}", value)),
        };

        Ok(command)
    }
}
