use anyhow::{anyhow, Result};
use rustyline::config::BellStyle;
use rustyline::{Config, DefaultEditor};

use crate::process::Process;

const HISTORY_FILE: &str = "~/.cache/jdb/history";

pub struct Debugger {
    line_reader: DefaultEditor,
    /// Flag if the program is currently being debugged
    debugging: bool,
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
            debugging: false,
        })
    }

    pub fn next(&mut self, process: &mut Process) -> Result<()> {
        let line = self.line_reader.readline("(jdb) ")?;
        if line.is_empty() {
            return Ok(());
        }

        println!("{:?}", line);
        
        let _ = self.line_reader.add_history_entry(line.as_str());

        let cmd = Command::try_from(line)?;
        self.dispatch_command(cmd, process)?;

        self.line_reader.append_history(HISTORY_FILE)?;

        Ok(())
    }

    fn dispatch_command(&mut self, command: Command, process: &mut Process) -> Result<()> {
        match command {
            Command::Run(args) => {
                process.attach(args)?;
                self.debugging = true;
            },
            Command::Continue => process.resume()?,
            Command::Quit => {
                process.destroy()?;
                self.debugging = false;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Run(Vec<String>),
    Continue,
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
            _ => return Err(anyhow!("unknown command: {:?}", value))
        };

        Ok(command)
    }
}

