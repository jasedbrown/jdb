use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use log::LevelFilter;
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
use std::{io, thread::JoinHandle, time::Duration};
use strum::{Display, EnumIter, FromRepr};
use tracing::{debug, error, trace};
use tui_logger::{TuiWidgetEvent, TuiWidgetState};

use crate::{JdbEvent, debugger::Debugger, process::Process, tui::render::render_screen};

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
    /// Current command being constructed in the minibuffer.
    command_input: String,
    /// Last response emitted after running a command (shown in echo area).
    last_command_response: Option<String>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        let panes = vec![
            DebuggerPane::Source,
            DebuggerPane::Locals,
            DebuggerPane::Logs,
            DebuggerPane::Command,
        ];

        DebuggerState {
            panes,
            focus_pane_idx: 3,
            command_input: String::new(),
            last_command_response: None,
        }
    }
}

impl DebuggerState {
    pub fn is_focus(&self, pane: &DebuggerPane) -> bool {
        let focus = self.panes.get(self.focus_pane_idx).unwrap_or_else(|| {
            panic!(
                "Array index {} out of bounds {}",
                self.focus_pane_idx,
                self.panes.len()
            )
        });
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

    fn push_input(&mut self, ch: char) {
        self.command_input.push(ch);
    }

    fn pop_input(&mut self) {
        self.command_input.pop();
    }

    fn current_command(&self) -> &str {
        &self.command_input
    }

    fn take_command(&mut self) -> String {
        let command = self.command_input.clone();
        self.command_input.clear();
        command
    }

    fn clear_last_command_response(&mut self) {
        self.last_command_response = None;
    }

    fn set_last_command_response(&mut self, message: impl Into<String>) {
        self.last_command_response = Some(message.into());
    }

    pub fn last_command_response(&self) -> Option<&str> {
        self.last_command_response.as_deref()
    }
}

pub struct DebuggerLogScreenState {
    states: Vec<TuiWidgetState>,
    current_pane_idx: usize,
}

impl Default for DebuggerLogScreenState {
    fn default() -> Self {
        // assume 4 widget panes in the debug logging screen, a la the tuii-logger example
        let states = vec![
            TuiWidgetState::new().set_default_display_level(LevelFilter::Info),
            TuiWidgetState::new().set_default_display_level(LevelFilter::Info),
            TuiWidgetState::new().set_default_display_level(LevelFilter::Info),
            TuiWidgetState::new().set_default_display_level(LevelFilter::Info),
        ];

        Self {
            states,
            current_pane_idx: 0,
        }
    }
}

impl DebuggerLogScreenState {
    pub fn focus_next_pane(&mut self, forward: bool) {
        self.current_pane_idx = next_index(self.states.len(), self.current_pane_idx, forward);
    }

    fn transition(&mut self, event: TuiWidgetEvent) {
        let widget_state = self.states.get(self.current_pane_idx).unwrap_or_else(|| {
            panic!(
                "Array index {} out of bounds {}",
                self.current_pane_idx,
                self.states.len()
            )
        });
        widget_state.transition(event);
    }

    pub fn current_state(&self) -> &TuiWidgetState {
        self.states.get(self.current_pane_idx).unwrap_or_else(|| {
            panic!(
                "Array index {} out of bounds {}",
                self.current_pane_idx,
                self.states.len()
            )
        })
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

/// Enum of the panes within the Locals pane.
#[allow(dead_code)]
#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum LocalsPaneMode {
    #[strum(to_string = "Variables")]
    Variables,
    #[strum(to_string = "GP Regs")]
    #[default]
    // TODO: default to variables, once I actually get to that point.
    GeneralPurposeRegisters,
    #[strum(to_string = "FP Regs")]
    FloatingPoointRegisters,
    #[strum(to_string = "Debug Regs")]
    DebugRegisters,
}

/// The central nexus of state of the various screens for the TUI.
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
    tui_thread: Option<JoinHandle<()>>,
}

pub enum EventResult {
    Normal,
    Editor { command: String },
    Quit,
}

impl Tui {
    /// Put terminal into raw mode + alternate screen
    pub fn new(tui_tx: Sender<JdbEvent>, shutdown_rx: Receiver<()>) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        let tui_thread = std::thread::spawn(move || await_event(tui_tx, shutdown_rx));

        Ok(Self {
            terminal,
            state: Default::default(),
            tui_thread: Some(tui_thread),
        })
    }

    /// Restore terminal from TUI state
    pub fn exit(&mut self) -> Result<()> {
        if let Some(handle) = self.tui_thread.take() {
            let _ = handle.join();
        }
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

    pub fn record_command_response(&mut self, message: impl Into<String>) {
        self.state
            .debugger_state
            .set_last_command_response(message.into());
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

    pub fn handle_key_press(&mut self, key: KeyEvent) -> Result<EventResult> {
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
    let in_edit_mode = state.in_edit_mode();
    trace!(?key, ?in_edit_mode, "Debugger screen key press");

    if in_edit_mode {
        // M-e is the magick binding to exit editor mode
        if key.code == KeyCode::Char('x') && key.modifiers == KeyModifiers::ALT {
            state.set_focus(&DebuggerPane::Source);
        } else {
            match key.code {
                // grab the current line before sending the RETURN event
                KeyCode::Enter => {
                    let command = state.take_command();
                    let command_is_empty = command.is_empty();
                    state.clear_last_command_response();

                    // preserve empty-command detection for downstream handling
                    if command_is_empty {
                        trace!("Editor command is empty, will replay last command");
                    }
                    ret_code = EventResult::Editor { command };
                }
                KeyCode::Backspace => {
                    state.pop_input();
                }
                KeyCode::Char(c) => {
                    state.push_input(c);
                }
                _ => {}
            }
        }
    } else {
        match key.code {
            KeyCode::Char(c) => match c {
                'x' if matches!(key.modifiers, KeyModifiers::META | KeyModifiers::ALT) => {
                    state.set_focus(&DebuggerPane::Command);
                }
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
    state: &mut DebuggerLogScreenState,
    key: KeyEvent,
) -> Result<EventResult> {
    trace!(?key, "Debug log screen key event");
    let mut ret_code = EventResult::Normal;
    match key.code {
        KeyCode::Char(c) => match c {
            // this is a development-time only, semi-sneaky back door to quit the debugger
            // if i've fucked up somehow ...
            'q' => ret_code = EventResult::Quit,
            ' ' => state.transition(TuiWidgetEvent::SpaceKey),
            '+' => state.transition(TuiWidgetEvent::PlusKey),
            '-' => state.transition(TuiWidgetEvent::MinusKey),
            'h' => state.transition(TuiWidgetEvent::HideKey),
            'f' => state.transition(TuiWidgetEvent::FocusKey),
            _ => {}
        },
        KeyCode::Tab => state.focus_next_pane(true),
        KeyCode::BackTab => state.focus_next_pane(false),
        KeyCode::Esc => state.transition(TuiWidgetEvent::EscapeKey),
        KeyCode::PageUp => state.transition(TuiWidgetEvent::PrevPageKey),
        KeyCode::PageDown => state.transition(TuiWidgetEvent::NextPageKey),
        KeyCode::Up => state.transition(TuiWidgetEvent::UpKey),
        KeyCode::Down => state.transition(TuiWidgetEvent::DownKey),
        KeyCode::Left => state.transition(TuiWidgetEvent::LeftKey),
        KeyCode::Right => state.transition(TuiWidgetEvent::RightKey),
        _ => {}
    }
    Ok(ret_code)
}

fn await_event(tui_tx: Sender<JdbEvent>, shutdown_rx: Receiver<()>) {
    loop {
        match crossterm::event::poll(Duration::from_millis(100)) {
            Ok(has_event) => {
                if has_event {
                    match crossterm::event::read() {
                        Ok(event) => match event {
                            Event::Key(key) => {
                                match key.kind {
                                    // we only care about key presses.
                                    KeyEventKind::Release | KeyEventKind::Repeat => {}
                                    KeyEventKind::Press => {
                                        if let Err(e) = tui_tx.send(JdbEvent::TerminalKey(key)) {
                                            error!("Error when sending to tui_tx channel: {:?}", e)
                                        }
                                    }
                                }
                            }
                            Event::Resize(_, _) => {
                                if let Err(e) = tui_tx.send(JdbEvent::TerminalResize) {
                                    error!("Error when sending to tui_tx channel: {:?}", e)
                                }
                            }
                            // handle Event::Paste
                            _ => {}
                        },
                        // TODO: might want to send an error type of JdbEvent
                        Err(e) => error!("Error reading terminal::event: {:?}", e),
                    }
                } else {
                    // check the shutdown channel
                    if !shutdown_rx.is_empty() {
                        // drain the channel
                        debug!("Terminal event reading thread received shutdown message");
                        return;
                    }
                }
            }
            // TODO: might want to send an error type of JdbEvent
            Err(e) => error!("Error polling for terminal event: {:?}", e),
        }
    }
}
