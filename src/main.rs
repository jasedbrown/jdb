use anyhow::Result;
use clap::Parser;
use jdb::{
    debugger::{Debugger, DispatchResult},
    options::Options,
    process::Process,
    tui::{EventResult, Tui},
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
    let mut tui = Tui::new()?;

    let mut mode = Mode::Tui;

    // start main loop here
    loop {
        // i *think* we want to render on every loop iteration ???
        tui.render(&debugger)?;
        match mode {
            Mode::Tui => {
                match tui.await_event() {
                    Ok(EventResult::Normal) => {
                        // ???
                    }
                    Ok(EventResult::FocusOnEditor) => {
                        // tui.suspend_tui()?;
                        mode = Mode::Edit;
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
            Mode::Edit => match debugger.next(&mut process) {
                Ok(DispatchResult::Normal) => {
                    // i think we want to redraw here (esp for moving forward in src, variable updating, ...)
                }
                Ok(DispatchResult::Exit) => {
                    break;
                }
                Ok(DispatchResult::SwitchToTui) => {
                    // tui.resume_tui()?;
                    mode = Mode::Tui;
                }
                Err(e) => println!("Error: {:?}", e),
            },
        }
    }

    tui.exit()?;
    Ok(())
}
