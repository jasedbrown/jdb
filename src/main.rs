use anyhow::Result;
use clap::Parser;
use jdb::{
    debugger::{Debugger, DispatchResult},
    options::Options,
    process::Process,
};

pub enum Mode {
    Tui,
    Edit,
}

fn main() -> Result<()> {
    let options = Options::parse();
    options.validate()?;

    let mut debugger = Debugger::new(&options)?;
    let mut process = Process::new(options);

    let mut mode = Mode::Edit;

    // init

    // start main loop here
    loop {
        match mode {
            Mode::Tui => {
                // noop
                mode = Mode::Edit;
            }
            Mode::Edit => match debugger.next(&mut process) {
                Ok(DispatchResult::Normal) => {
                    // i think we want to redraw here (esp for moving forward in src, variable updating, ...)
                }
                Ok(DispatchResult::Exit) => {
                    break;
                }
                Ok(DispatchResult::SwitchToTui) => {
                    mode = Mode::Tui;
                }
                Err(e) => println!("Error: {:?}", e),
            },
        }
    }

    Ok(())
}
