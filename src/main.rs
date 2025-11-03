use anyhow::Result;
use clap::Parser;
use jdb::{debugger::{Debugger, DispatchResult}, options::Options, process::Process};

fn main() -> Result<()> {
    let options = Options::parse();
    options.validate()?;

    let mut debugger = Debugger::new(&options)?;
    let mut process = Process::new(options);

    enum AppState{
        Running,
        Stopped,
    };
    let mut cur_state = AppState::Stopped;
    
    // start main loop here
    loop {
        match cur_state{
            AppState::Running => {
                // render the tui
                // render_tui(&state);                
            }
            AppState::Stopped => {
//                suspend_tui();                 // disable_raw + leave alt


                match debugger.next(&mut process) {
                    Ok(DispatchResult::Normal) => {},
                    Ok(DispatchResult::Exit) => {
                        break;
                    }
                    Err(e) => println!("Error: {:?}", e),
                }                
            }
        }
    }

    Ok(())
}
