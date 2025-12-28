use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};

/// Configuration to enable or disable linux ASLR on the inferior processes.
#[derive(Copy, Clone, Debug)]
pub enum Aslr {
    Enabled,
    Disabled,
}

/// Basic CLI options for the debugger.
#[derive(Clone, Debug)]
pub struct Options {
    pub executable: PathBuf,
}

impl Options {
    /// Parse options from the current process' CLI arguments.
    pub fn from_env() -> Result<Self> {
        Self::from_args(env::args().skip(1))
    }

    /// Parse options from an iterator of strings (for tests).
    pub fn from_args<I, S>(mut args: I) -> Result<Self>
    where
        I: Iterator<Item = S>,
        S: Into<String>,
    {
        let executable = args
            .next()
            .map(|s| s.into())
            .ok_or_else(|| anyhow!("expected executable path as first argument"))?;

        let options = Options {
            executable: PathBuf::from(executable),
        };
        options.validate()?;
        Ok(options)
    }

    pub fn validate(&self) -> Result<()> {
        if self.executable.as_os_str().is_empty() {
            return Err(anyhow!("executable path must not be empty"));
        }

        if !self.executable.exists() {
            return Err(anyhow!("executable does not exist: {:?}", self.executable));
        }

        Ok(())
    }
}
