use anyhow::{Result, anyhow};
use ratatui::{
    Terminal,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io;
use tracing::trace;
use tui_textarea::TextArea;

use crate::{debugger::Debugger, process::Process, tui::render::render_screen};

mod render;

fn next_index(len: usize, cur_idx: usize, increment: bool) -> usize {
    if increment {
        (cur_idx + 1) % len
    } else if cur_idx == 0 {
        len - 1
    } else {
        cur_idx - 1
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum DebuggerPane {
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
pub struct DebuggerState {
    /// The current panes in the TUI
    panes: Vec<DebuggerPane>,
    /// The currently focued pane.
    focus_pane_idx: usize,
    /// The command editor. Since this is a stateful `Widget`, as well as being
    /// the most important Widget in this damn debugger, we keep a long-lived
    /// instance here.
    // TODO: the static lifetime might be wrong/bullshit ...
    editor: TextArea<'static>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        let panes = vec![
            DebuggerPane::Source,
            DebuggerPane::Locals,
            DebuggerPane::Command,
            DebuggerPane::Logs,
        ];

        let textarea = TextArea::default();

        DebuggerState {
            panes,
            focus_pane_idx: 0,
            editor: textarea,
        }
    }
}

impl DebuggerState {
    pub fn is_focus(&self, pane: &DebuggerPane) -> bool {
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

    pub fn focus_next_pane(&mut self, forward: bool) {
        self.focus_pane_idx = next_index(self.panes.len(), self.focus_pane_idx, forward);
    }

    fn set_focus(&mut self, pane: &DebuggerPane) {
        for (i, p) in self.panes.iter().enumerate() {
            if p == pane {
                self.focus_pane_idx = i;
                return;
            }
        }
        unreachable!("Should have found pane type {:?} in current panes", pane);
    }

    fn in_edit_mode(&self) -> bool {
        self.is_focus(&DebuggerPane::Command)
    }
}

#[derive(Debug, Default)]
pub struct DebuggerLogScreenState {
    // tui_logger ??? not sure if i need a long-lived reference
    current_pane_idx: usize,
}

impl DebuggerLogScreenState {
    pub fn focus_next_pane(&mut self, forward: bool) {
        self.current_pane_idx = next_index(2, self.current_pane_idx, forward);
    }
}

/// Enum of the primary screens available in `jdb`
#[derive(Copy, Clone, Debug)]
pub enum ScreenMode {
    /// The primary screen for debugging activities
    MainDebugger,
    /// See the logs from `jdb` itself.
    DebuggerLogging,
}

/// The central nexus of state of the various screens for the TUI.
#[derive(Debug)]
struct TuiState {
    debugger_state: DebuggerState,
    logging_state: DebuggerLogScreenState,

    /// The current screen that should be displayed/interacted with.
    screen_mode: ScreenMode,
}

impl Default for TuiState {
    fn default() -> Self {
        TuiState {
            debugger_state: Default::default(),
            logging_state: Default::default(),
            screen_mode: ScreenMode::MainDebugger,
        }
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
            .draw(|frame| render_screen(&self.state, debugger, process, frame))
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

    fn handle_function_key(&mut self, fkey_num: u8) -> Result<EventResult> {
        // TODO: might need to swap/store some additional state. perhaps if we were in
        // the editor mode, something might need to be stashed (not really sure)??

        let cur_screen = self.state.screen_mode;
        match fkey_num {
            1 => self.state.screen_mode = ScreenMode::MainDebugger,
            2 => self.state.screen_mode = ScreenMode::DebuggerLogging,
            _ => {} // ignore other keys
        }

        trace!(prev_screen = ?cur_screen, next_screen = ?self.state.screen_mode, "Changing screen mode");

        Ok(EventResult::Normal)
    }

    fn handle_key_press(&mut self, key: KeyEvent) -> Result<EventResult> {
        // let mut ret_code = EventResult::Normal;

        // handle Fn keys before everything as that will switch screens
        if let KeyCode::F(fkey_num) = key.code {
            return self.handle_function_key(fkey_num);
        }

        match self.state.screen_mode {
            ScreenMode::MainDebugger => {
                debugger_screen_key_press(&mut self.state.debugger_state, key)
            }
            ScreenMode::DebuggerLogging => {
                logging_screen_key_press(&mut self.state.logging_state, key)
            }
        }
    }
}

fn debugger_screen_key_press(state: &mut DebuggerState, key: KeyEvent) -> Result<EventResult> {
    let mut ret_code = EventResult::Normal;

    if state.in_edit_mode() {
        // M-e is the magick binding to exit editor mode
        if key.code == KeyCode::Char('e') && key.modifiers == KeyModifiers::META {
            state.set_focus(&DebuggerPane::Source);
        } else {
            let is_enter = key.code == KeyCode::Enter;
            assert!(state.editor.input(key));

            if is_enter {
                let lines = state.editor.lines();
                let last_line = lines[lines.len() - 1].clone();
                ret_code = EventResult::Editor { command: last_line };
            }
        }
    } else {
        match key.code {
            KeyCode::Char(c) => match c {
                'c' | 'e' => {
                    state.set_focus(&DebuggerPane::Command);
                }
                's' => {
                    state.set_focus(&DebuggerPane::Source);
                }
                'l' => {
                    state.set_focus(&DebuggerPane::Locals);
                }
                'o' => {
                    state.set_focus(&DebuggerPane::Logs);
                }
                'q' => ret_code = EventResult::Quit,
                _ => {}
            },
            KeyCode::Tab => state.focus_next_pane(true),
            KeyCode::BackTab => state.focus_next_pane(false),
            _ => {}
        }
    }

    Ok(ret_code)
}

fn logging_screen_key_press(
    _state: &mut DebuggerLogScreenState,
    _key: KeyEvent,
) -> Result<EventResult> {
    let ret_code = EventResult::Normal;
    Ok(ret_code)
}
