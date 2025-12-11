use std::fs::exists;
use std::path::PathBuf;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Result, anyhow};

use crate::options::Options;

pub struct CommandHistory {
    /// Resolved (absolute) path to the history file.
    history_file: PathBuf,
    history: Vec<String>,
}

impl CommandHistory {
    pub fn new(cli_options: &Options) -> Result<Self> {
        let history_file = resolve_history_file(&cli_options.history_file)?;

        // let config = Config::builder()
        //     .edit_mode(rustyline::EditMode::Emacs)
        //     .max_history_size(10000)?
        //     .history_ignore_dups(true)?
        //     .bell_style(BellStyle::None)
        //     .tab_stop(4)
        //     .build();
        // let mut line_reader = DefaultEditor::with_config(config)?;

        // line_reader.bind_sequence(
        //     KeyEvent(KeyCode::Char('e'), Modifiers::ALT),
        //     EventHandler::Simple(Cmd::Interrupt),
        // );

        let history = read_history(&history_file)?;

        Ok(Self {
            history_file,
            history,
        })
    }

    pub fn add(&mut self, cmd: &str) -> Result<()> {
        let should_append = match self.history.last() {
            Some(last) if *last != cmd => true,
            None => true,
            Some(_) => false,
        };

        if should_append {
            self.history.push(cmd.to_string());

            // TODO: append to history_file and flush to disk
        }

        Ok(())
    }
}

fn read_history(history_file: &PathBuf) -> Result<Vec<String>> {
    if exists(history_file)? {
        let file = File::open(history_file)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            lines.push(line?);
        }
        tracing::trace!("history: loaded {:?} entries", lines.len());
        Ok(lines)
    } else {
        Ok(Vec::new())
    }
}

fn resolve_history_file(history_file: &Option<PathBuf>) -> Result<PathBuf> {
    let mut path = match history_file {
        Some(p) => p.clone(),
        None => {
            let cache_dir = env::var_os("XDG_CACHE_HOME")
                .and_then(|p| {
                    if p.is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(p))
                    }
                })
                .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
                .ok_or_else(|| anyhow!("Neither XDG_CACHE_HOME nor HOME is set"))?;
            cache_dir.join("jdb").join("history")
        }
    };

    if let Some(s) = path.to_str()
        && s.starts_with("~/")
    {
        let home = env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
        path = PathBuf::from(home).join(&s[2..]);
    }

    Ok(path)
}
