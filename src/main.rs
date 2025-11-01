use anyhow::Result;
use clap::Parser;
use jdb::{debugger::{Debugger, DispatchResult}, options::Options, process::Process};

fn main() -> Result<()> {
    let options = Options::parse();
    options.validate()?;

    let mut debugger = Debugger::new(&options)?;
    let mut process = Process::new(options);

    // start main loop here
    loop {
        match debugger.next(&mut process) {
            Ok(DispatchResult::Normal) => {},
            Ok(DispatchResult::Exit) => {
                break;
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}
