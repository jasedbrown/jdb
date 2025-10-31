use anyhow::Result;
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
        println!("{:?}", line);

        // history mgmt
        let _ = self.line_reader.add_history_entry(line.as_str());
        self.line_reader.append_history(HISTORY_FILE)?;

        // assume 'continue' and wait for the inferior process
        process.resume()?;

        Ok(())
    }
}
