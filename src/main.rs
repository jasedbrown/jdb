use anyhow::Result;
use clap::Parser;
use jdb::{
    debugger::{Debugger, DispatchResult},
    options::Options,
    process::Process,
    tui::{EventResult, Tui},
};

fn main() -> Result<()> {
    let options = Options::parse();
    options.validate()?;

    let mut debugger = Debugger::new(&options)?;
    let mut process = Process::new(options);
    let mut tui = Tui::new()?;

    loop {
        // i *think* we want to render on every loop iteration ???
        tui.render(&debugger, &process)?;
        match tui.await_event() {
            Ok(EventResult::Normal) => {
                // nop?
            }
            Ok(EventResult::Editor { command }) => {
                match debugger.next(command, &mut process) {
                    Ok(DispatchResult::Normal) => {
                        // i think we want to redraw here (esp for moving forward in src, variable updating, ...)
                    }
                    Ok(DispatchResult::Exit) => {
                        break;
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            Ok(EventResult::Quit) => {
                // If i actually allow this from the TUI, need to stop debugger/inferior process
                break;
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    tui.exit()?;
    Ok(())
}
