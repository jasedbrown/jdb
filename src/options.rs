use std::path::PathBuf;

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
        // Arguments to the process
        #[arg(long = "args")]
        args: String,
    },
}

#[derive(Clone, Debug, Parser)]
#[command(version, about = "JDB (jason's debugger)")]
pub struct Options {
    #[command(subcommand)]
    pub launch_type: LaunchType,
}
