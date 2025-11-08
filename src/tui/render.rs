use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    debugger::Debugger,
    process::Process,
    tui::{DebuggerPane, TuiState},
};

fn build_watchers_pane(state: &TuiState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Locals, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_editor_pane(state: &TuiState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Command, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_output_pane(state: &TuiState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Logs, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_source_pane(state: &TuiState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Source, None, state);
    Paragraph::new("...")
        .style(Style::default().fg(Color::Green))
        .block(block)
}

fn build_bounding_rect<'a>(
    pane: &DebuggerPane,
    name_override: Option<String>,
    state: &TuiState,
) -> Block<'a> {
    let is_focus = state.is_focus(pane);
    let mut style = Style::default();
    if is_focus {
        style = style.bold().blue();
    }
    let title = Line::from(format!(
        " {} ",
        name_override.unwrap_or(pane.display_name())
    ))
    .style(style);

    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title.left_aligned());
    if is_focus {
        block = block.border_set(border::DOUBLE).blue();
    }
    block
}

pub fn render_inner(state: &TuiState, _debugger: &Debugger, _process: &Process, frame: &mut Frame) {
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
    frame.render_widget(command_pane, bottom_pane_chunks[0]);
    // logs/stdout pane
    let output_pane = build_output_pane(state);
    frame.render_widget(output_pane, bottom_pane_chunks[1]);
}
