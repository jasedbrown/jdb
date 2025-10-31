use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Subcommand)]
pub enum LaunchType {
    // Attach to an already executing inferior process.
    Pid {
        // PID of an existing process
        #[arg(short = 'p', long = "pid")]
        pid: i32,
    },
    // Launch and attach to an inferior process.
    Name {
        // Path to process executable
        #[arg(short = 'n', long = "name")]
        name: PathBuf,
    },
}

impl LaunchType {
    /// Should the inferior process be terminated when debugging is complete?
    pub fn terminate_on_exit(&self) -> bool {
        match self {
            LaunchType::Pid { .. } => false,
            LaunchType::Name { .. } => true,
        }
    }
}

#[derive(Clone, Debug, Parser)]
#[command(version, about = "JDB (jason's debugger)")]
pub struct Options {
    #[command(subcommand)]
    pub launch_type: LaunchType,
}

impl Options {
    pub fn validate(&self) -> Result<()> {
        match self.launch_type {
            LaunchType::Pid { pid } => {
                if pid <= 0 {
                    return Err(anyhow!("PID must be greater than zero: {:?}", pid));
                }
            }
            LaunchType::Name { .. } => {}
        }

        Ok(())
    }
}
