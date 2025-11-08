use anyhow::{Result, anyhow};
use ratatui::{
    Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io;

use crate::{debugger::Debugger, process::Process, tui::render::render_inner};

mod render;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[allow(dead_code)]
enum DebuggerPane {
    Assembly,
    Breakpoints,
    Command,
    Locals,
    Logs,
    Source,
    Watchpoints,
}

impl DebuggerPane {
    fn display_name(&self) -> String {
        use DebuggerPane::*;
        let name = match self {
            Assembly => "assembly",
            Breakpoints => "breakpoints",
            Command => "command",
            Locals => "locals",
            Logs => "logs",
            Source => "source",
            Watchpoints => "watchpoints",
        };
        name.to_string()
    }
}

#[derive(Debug)]
struct TuiState {
    panes: Vec<DebuggerPane>,
    focus_pane_idx: usize,
}

impl Default for TuiState {
    fn default() -> Self {
        let panes = vec![
            DebuggerPane::Source,
            DebuggerPane::Locals,
            DebuggerPane::Command,
            DebuggerPane::Logs,
        ];
        TuiState {
            panes,
            focus_pane_idx: 0,
        }
    }
}

impl TuiState {
    fn is_focus(&self, pane: &DebuggerPane) -> bool {
        let focus = self.panes.get(self.focus_pane_idx).expect(
            format!(
                "Array index {} out of bounds {}",
                self.focus_pane_idx,
                self.panes.len()
            )
            .as_str(),
        );
        focus == pane
    }

    fn focus_next_pane(&mut self, forward: bool) -> DebuggerPane {
        let idx = if forward {
            (self.focus_pane_idx + 1) % self.panes.len()
        } else if self.focus_pane_idx == 0 {
            self.panes.len() - 1
        } else {
            self.focus_pane_idx - 1
        };
        self.focus_pane_idx = idx;

        *self.panes.get(self.focus_pane_idx).unwrap()
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: TuiState,
}

pub enum EventResult {
    Normal,
    Editor { command: String },
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

        Ok(Self {
            terminal,
            state: Default::default(),
        })
    }

    /// Restore terminal from TUI state
    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Render the TUI
    pub fn render(&mut self, debugger: &Debugger, process: &Process) -> Result<()> {
        match self
            .terminal
            .draw(|frame| render_inner(&self.state, debugger, process, frame))
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }

    pub fn await_event(&mut self) -> Result<EventResult> {
        match crossterm::event::read() {
            Ok(event) => match event {
                Event::Key(key) => {
                    match key.kind {
                        // we only care about key presses.
                        KeyEventKind::Release | KeyEventKind::Repeat => Ok(EventResult::Normal),
                        KeyEventKind::Press => self.handle_key_press(key),
                    }
                }
                Event::Resize(_, _) => Ok(EventResult::Normal),
                _ => Ok(EventResult::Normal),
            },
            Err(e) => Err(anyhow!(e)),
        }
    }

    fn set_focus(&mut self, pane: &DebuggerPane) {
        for (i, p) in self.state.panes.iter().enumerate() {
            if p == pane {
                self.state.focus_pane_idx = i;
                return;
            }
        }
        unreachable!("Should have found pane type {:?} in current panes", pane);
    }

    fn handle_tab(&mut self, forward: bool) {
        let next_pane = self.state.focus_next_pane(forward);
        self.set_focus(&next_pane);
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> Result<EventResult> {
        let mut ret_code = EventResult::Normal;
        match key.code {
            KeyCode::Char(c) => match c {
                'c' | 'e' => {
                    self.set_focus(&DebuggerPane::Command);
                }
                's' => {
                    self.set_focus(&DebuggerPane::Source);
                }
                'l' => {
                    self.set_focus(&DebuggerPane::Locals);
                }
                'o' => {
                    self.set_focus(&DebuggerPane::Logs);
                }
                'q' => ret_code = EventResult::Quit,
                _ => {}
            },
            KeyCode::Tab => self.handle_tab(true),
            KeyCode::BackTab => self.handle_tab(false),
            _ => {}
        };

        Ok(ret_code)
    }
}
