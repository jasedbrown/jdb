use anyhow::{Result, anyhow};
use crossbeam_channel::{select, unbounded};
use jdb::{
    JdbEvent,
    debugger::{Debugger, DispatchResult},
    options::Options,
    process::Process,
    tui::{EventResult, Tui},
};
use tracing::{error, trace};
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{env, fs, path::PathBuf};

fn init_logging() -> Result<WorkerGuard> {
    // Layer 1: send tracing events to tui-logger’s widget
    tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Trace);
    let tui_layer = tui_logger::TuiTracingSubscriberLayer;

    let state_dir = env::var_os("XDG_STATE_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .ok_or_else(|| anyhow!("Neither XDG_STATE_HOME nor HOME is set"))?;
    let log_dir = state_dir.join("jdb");

    // Layer 2: send tracing events to file appender
    // ensure log dir exists
    fs::create_dir_all(&log_dir)?;
    let file_appender = tracing_appender::rolling::never(&log_dir, "jdb.log");

    // TODO: it would be fancy to add a heaer to the log file every time the
    // debugger is launched (maybe even with config options ...)

    let (file_writer, guard) = non_blocking(file_appender);
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false) // files usually without ANSI
        .with_writer(file_writer);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(tui_layer)
        .init();

    Ok(guard)
}

fn main() -> Result<()> {
    let options = Options::from_env()?;

    let _guard = init_logging()?;

    let mut debugger = Debugger::new()?;

    let (process_tx, process_rx) = unbounded();
    let (process_shutdown_tx, process_shutdown_rx) = unbounded();
    let mut process = Process::new(options, process_tx, process_shutdown_rx);

    let (tui_tx, tui_rx) = unbounded();
    let (tui_shutdown_tx, tui_shutdown_rx) = unbounded();
    let mut tui = Tui::new(tui_tx, tui_shutdown_rx)?;

    loop {
        tui.render(&debugger, &process)?;
        select! {
            // handle output from the inferior process
            recv(process_rx) -> msg => match msg {
                Ok(s) => process.receive_inferior_logging(s),
                Err(e) => error!("Error receiving message from inferior processing logging: {:?}", e),
            },
            // handle key presses
            recv(tui_rx) -> msg => match msg {
                Ok(jdb_event) => match jdb_event {
                    JdbEvent::TerminalKey(key_event) => {
                        match tui.handle_key_press(key_event) {
                            Ok(EventResult::Normal) => {
                                // nop?
                            }
                            Ok(EventResult::Editor { command }) => {
                                trace!(?command, "next editor command");
                                match debugger.next(command, &mut process) {
                                    Ok(DispatchResult::Normal) => {
                                        // i think we want to redraw here (esp for moving forward in src, variable updating, ...)
                                    }
                                    Ok(DispatchResult::Exit) => {
                                        tui.record_command_response("exiting debugger");
                                        break;
                                    }
                                    Err(e) => {
                                        tui.record_command_response(format!("error: {e}"));
                                        error!("Error: {:?}", e)
                                    }
                                }
                            }
                            Ok(EventResult::Quit) => {
                                // If i actually allow this from the TUI, need to stop debugger/inferior process
                                break;

                            },
                            Err(e) => error!("Error received from tui message channel: {:?}", e),
                        }
                    },
                    JdbEvent::TerminalResize => {}
                }
                Err(e) => error!("Error receiving message from inferior processing logging: {:?}", e),
            }
        }
    }

    // if we've exited the main loop, make sure to signal everyone to shutdown
    let _ = tui_shutdown_tx.send(());
    let _ = process_shutdown_tx.send(());

    tui.exit()?;
    Ok(())
}
