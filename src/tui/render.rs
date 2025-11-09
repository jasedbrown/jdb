use ratatui::{
    layout::{Constraint, Direction, Layout, Rect}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Tabs, Widget}, Frame
};
use tui_textarea::TextArea;

use crate::{
    debugger::Debugger,
    process::Process,
    tui::{DebuggerLogScreenState, DebuggerPane, DebuggerState, ScreenMode, TuiState},
};

fn build_watchers_pane(state: &DebuggerState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Locals, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_editor_pane(state: &DebuggerState) -> TextArea<'_> {
    let block = build_bounding_rect(&DebuggerPane::Command, None, state);
    // TODO: the clone() sucks, but let's get this app going, and then fix up
    let mut e = state.editor.clone();
    e.set_block(block);
    e
}

fn build_output_pane(state: &DebuggerState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Logs, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_source_pane(state: &DebuggerState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Source, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_bounding_rect<'a>(
    pane: &DebuggerPane,
    name_override: Option<String>,
    state: &DebuggerState,
) -> Block<'a> {
    let is_focus = state.is_focus(pane);
    let mut style = Style::default().green();
    if is_focus {
        style = style.bold().blue();
    }
    let title = Line::from(format!(
        " {} ",
        name_override.unwrap_or(pane.display_name())
    ))
    .style(style);

    let mut block = Block::default()
        .green()
        .borders(Borders::ALL)
        .title(title.left_aligned());
    if is_focus {
        block = block.border_set(border::DOUBLE).blue();
    }
    block
}

fn render_debugger_screen(
    state: &DebuggerState,
    _debugger: &Debugger,
    _process: &Process,
    frame: &mut Frame,
    rect: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rect);

    ///////////////////////////////
    // build top chunk (source and variable panes ...)
    let top_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(chunks[0]);

    // source pane
    let source_pane = build_source_pane(state);
    frame.render_widget(source_pane, top_pane_chunks[0]);
    // pane with locals / other ...
    let others_pane = build_watchers_pane(state);
    frame.render_widget(others_pane, top_pane_chunks[1]);

    /////////////////////////////
    // build bottom chunk (command and stdout panes ...)
    let bottom_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);
    // command pane
    let command_pane = build_editor_pane(state);
    frame.render_widget(&command_pane, bottom_pane_chunks[0]);
    // logs/stdout pane
    let output_pane = build_output_pane(state);
    frame.render_widget(output_pane, bottom_pane_chunks[1]);
}

fn render_logging_screen(
    _state: &DebuggerLogScreenState,
    _debugger: &Debugger,
    _process: &Process,
    _frame: &mut Frame,
    _rect: Rect,
) {
}

fn render_header(screen_mode: ScreenMode, frame: &mut Frame, rect: Rect) {
    let screens = ["Main (F1)", "Debugger logging (F2)"];

    let selected_idx = match screen_mode {
        ScreenMode::MainDebugger => 0,
        ScreenMode::DebuggerLogging => 1,
    };
    
    let tabs = screens
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect::<Tabs>()
        .block(Block::bordered().title("jdb"))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(selected_idx);
    frame.render_widget(tabs, rect);
}

pub fn render_screen(state: &TuiState, debugger: &Debugger, process: &Process, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    // draw header row
    render_header(state.screen_mode, frame, chunks[0]);

    match state.screen_mode {
        ScreenMode::MainDebugger => {
            render_debugger_screen(&state.debugger_state, debugger, process, frame, chunks[1])
        }
        ScreenMode::DebuggerLogging => {
            render_logging_screen(&state.logging_state, debugger, process, frame, chunks[1])
        }
    }
}
