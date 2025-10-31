use anyhow::Result;
use clap::Parser;
use jdb::{options::Options, Debugger, Process};

fn main() -> Result<()> {
    let options = Options::parse();
    let mut debugger = Debugger::new()?;
    let mut process = Process::new(options);
    

    // start main loop here

    process.attach()?;


    Ok(())
}
