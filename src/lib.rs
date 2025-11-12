use ratatui::crossterm::event::KeyEvent;

pub mod debugger;
pub mod options;
pub mod process;
pub mod tui;

pub enum JdbEvent {
    TerminalKey(KeyEvent),
    TerminalResize,
}
