use anyhow::Result;
use clap::Parser;
use jdb::{debugger::Debugger, options::Options, process::Process};

fn main() -> Result<()> {
    let options = Options::parse();
    options.validate()?;

    let mut debugger = Debugger::new()?;
    let mut process = Process::new(options);

    // start main loop here

    process.attach()?;

    Ok(())
}
