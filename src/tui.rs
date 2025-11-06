use anyhow::{anyhow, Result};
use ratatui::{
    crossterm::{
        self, event::{Event, KeyCode, KeyEvent, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
    }, prelude::CrosstermBackend, Terminal
};
use std::io;

use crate::debugger::Debugger;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

pub enum EventResult {
    Normal,
    FocusOnEditor,
    Quit,
}

impl Tui {
    /// Put terminal into raw mode + alternate screen
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Restore terminal from TUI state
    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Used right before running editor
    #[allow(dead_code)]
    fn suspend_tui() -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Used right after switching from editor
    #[allow(dead_code)]
    fn resume_tui() -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(())
    }

    /// Render the TUI
    pub fn render(&mut self, _debugger: &Debugger) -> Result<()> {
        Ok(())
    }

    pub fn await_event(&mut self) -> Result<EventResult> {
        match crossterm::event::read() {
            Ok(event) => match event {
                Event::Key(key) => {
                    match key.kind {
                        // we only care about key presses.
                        KeyEventKind::Release | KeyEventKind::Repeat => {
                            Ok(EventResult::Normal)
                        }
                        KeyEventKind::Press => {
                            self.handle_key_press(key)
                        }
                    }
                }
                Event::Resize(_, _) => {
                    Ok(EventResult::Normal)
                }
                _ => {
                    Ok(EventResult::Normal)
                }
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> Result<EventResult> {
        let ret_code = match key.code {
            KeyCode::Char(c) => match c {
                'e' => EventResult::FocusOnEditor,
                'q' => EventResult::Quit,
                _ => EventResult::Normal,
            }
            _ => EventResult::Normal
        };

        Ok(ret_code)
    }
}
