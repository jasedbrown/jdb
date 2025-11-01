use std::path::PathBuf;

use anyhow::{anyhow, Result};
use rustyline::config::BellStyle;
use rustyline::{Config, DefaultEditor};

use crate::options::Options;
use crate::process::Process;

const HISTORY_FILE: &str = "~/.cache/jdb/history";

pub struct Debugger {
    line_reader: DefaultEditor,
    /// Flag if the program is currently being debugged.
    debugging: bool,
    /// Resolved (absolute) path to the history file.
    history_file: PathBuf,
}

impl Debugger {
    pub fn new(cli_options: &Options) -> Result<Debugger> {
        let config = Config::builder()
            .edit_mode(rustyline::EditMode::Emacs)
            .max_history_size(10000)?
            .bell_style(BellStyle::None)
            .tab_stop(4)
            .build();
        let mut line_reader = DefaultEditor::with_config(config)?;

        let history_file = resolve_history_file(&cli_options.history_file)?;
        let _ = line_reader.load_history(&history_file);

        Ok(Debugger {
            line_reader,
            debugging: false,
            history_file,
        })
    }

    pub fn next(&mut self, process: &mut Process) -> Result<DispatchResult> {
        let line = self.line_reader.readline("(jdb) ")?;
        if line.is_empty() {
            return Ok(DispatchResult::Normal);
        }

        let _ = self.line_reader.add_history_entry(line.as_str());

        let cmd = Command::try_from(line)?;
        let result = self.dispatch_command(cmd, process)?;

        self.line_reader.append_history(&self.history_file)?;

        Ok(result)
    }

    fn dispatch_command(&mut self, command: Command, process: &mut Process) -> Result<DispatchResult> {
        let mut res = DispatchResult::Normal;
        match command {
            Command::Run(args) => {
                process.attach(args)?;
                self.debugging = true;
            },
            Command::Continue => {
                process.resume()?;
                process.wait_on_signal()?;
            }
            Command::Quit => {
                process.destroy()?;
                self.debugging = false;
                res = DispatchResult::Exit;
            }
        }

        Ok(res)
    }
}

fn resolve_history_file(history_file: &Option<PathBuf>) -> Result<PathBuf> {
    let file_name = match &history_file {
        Some(s) => s.to_str().expect("Must be a valid path"),
        None => HISTORY_FILE,
    };

    let fname = if file_name.starts_with("~/") {
        dirs::home_dir().map(|home| home.join(&file_name[2..]))
    } else {
        PathBuf::from(file_name).into()
    };

    Ok(fname.expect("Must have resolved hsitory file"))
}

#[derive(Clone, Debug)]
pub enum DispatchResult {
    Normal,
    Exit,
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

