use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(version, about = "JDB (jason's debugger)")]
pub struct Options {
    pub executable: PathBuf,
    #[arg(long, short = 'p', required = false)]
    pub pid: Option<i32>,
    #[arg(long, required = false)]
    pub history_file: Option<PathBuf>,
}

impl Options {
    pub fn validate(&self) -> Result<()> {
        if let Some(pid) = self.pid
            && pid <= 0
        {
            return Err(anyhow!("PID must be greater than zero: {:?}", pid));
        }
        Ok(())
    }
}
