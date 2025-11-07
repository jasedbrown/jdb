use anyhow::{Result, anyhow};
use ratatui::{
    Frame, Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io;

use crate::{debugger::Debugger, process::Process};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
    fn is_focus(&self, pane: &DebuggerPane) -> Result<bool> {
        let focus = match self.panes.get(self.focus_pane_idx) {
            Some(p) => p,
            None => {
                return Err(anyhow!("Array index {} out of bounds {}", self.focus_pane_idx, self.panes.len()));
            }
        };
        Ok(focus == pane)
    }

    fn focus_next_pane(&mut self, forward: bool) -> DebuggerPane {
        let idx = if forward {
            let mut i = self.focus_pane_idx + 1;
            if i >= self.panes.len() {
                i = 0;
            }
            i
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
    pub fn render(&mut self, debugger: &Debugger, process: &Process) -> Result<()> {
        match self
            .terminal
            .try_draw(|frame| {
                match render_inner(&self.state, debugger, process, frame) {
                    Ok(w) => Ok(w),
                    Err(e) => Err(io::Error::other(e)),
                }
            })
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

    fn handle_tab(&mut self, forward: bool) -> Result<EventResult> {
        let next_pane = self.state.focus_next_pane(forward);
        self.set_focus(&next_pane);
        let result = if matches!(next_pane, DebuggerPane::Command) {
            EventResult::FocusOnEditor                    
        } else {
            EventResult::Normal
        };

        Ok(result)
    }
    
    fn handle_key_press(&mut self, key: KeyEvent) -> Result<EventResult> {
        let mut ret_code = EventResult::Normal;
        match key.code {
            KeyCode::Char(c) => match c {
                'c' | 'e' => {
                    self.set_focus(&DebuggerPane::Command);
                    ret_code = EventResult::FocusOnEditor;
                }
                's' => {
                    self.set_focus(&DebuggerPane::Source);
                    ret_code = EventResult::Normal;
                }
                'l' => {
                    self.set_focus(&DebuggerPane::Locals);
                    ret_code = EventResult::Normal;
                }
                'o' => {
                    self.set_focus(&DebuggerPane::Logs);
                    ret_code = EventResult::Normal;
                }
                'q' => ret_code = EventResult::Quit,
                _ => {},
            },
            KeyCode::Tab => ret_code = self.handle_tab(true)?,
            KeyCode::BackTab => ret_code = self.handle_tab(false)?,
            _ => {},
        };

        Ok(ret_code)
    }
}

fn build_pane(pane: &DebuggerPane, state: &TuiState) -> Result<impl Widget> {
    let is_focus = state.is_focus(pane)?;
    let mut style = Style::default();
    if is_focus {
        style = style.bold().blue();
    }
    let title = Line::from(format!(" {} ", pane.display_name())).style(style);

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title.left_aligned());
    if is_focus {
        block = block.border_set(border::DOUBLE).blue();
    }

    let widget = Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block);

    Ok(widget)
}

fn render_inner(state: &TuiState, _debugger: &Debugger, _process: &Process, frame: &mut Frame) -> Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(frame.area());

    ///////////////////////////////
    // build top chunk (source and variable panes ...)
    let top_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[0]);

    // source pane
    let source_pane = build_pane(&DebuggerPane::Source, state)?;
    frame.render_widget(source_pane, top_pane_chunks[0]);
    // pane with locals / other ...
    let others_pane = build_pane(&DebuggerPane::Locals, state)?;
    frame.render_widget(others_pane, top_pane_chunks[1]);

    /////////////////////////////
    // build bottom chunk (command and stdout panes ...)
    let bottom_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);
    // command pane
    let command_pane = build_pane(&DebuggerPane::Command, state)?;
    frame.render_widget(command_pane, bottom_pane_chunks[0]);
    // logs/stdout pane
    let output_pane = build_pane(&DebuggerPane::Logs, state)?;
    frame.render_widget(output_pane, bottom_pane_chunks[1]);

    Ok(())
}
