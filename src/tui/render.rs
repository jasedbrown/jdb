use log::LevelFilter;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};
use tui_logger::{
    LogFormatter, TuiLoggerLevelOutput, TuiLoggerSmartWidget, TuiLoggerWidget, TuiWidgetState,
};

use crate::{
    debugger::Debugger,
    process::Process,
    tui::{DebuggerLogScreenState, DebuggerPane, DebuggerState, ScreenMode, TuiState},
};

/// This pane will render the local variables, and various registers.
fn build_watchers_pane(state: &TuiState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Locals, None, &state.debugger_state);
    Paragraph::new("x: 42").block(block)
}

fn build_command_pane(state: &DebuggerState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Command, Some("command".to_string()), state);
    let prompt = Span::styled("jdb> ", Style::default().fg(Color::Cyan).bold());
    let input =
        Span::raw(state.current_command().to_string()).style(Style::default().fg(Color::White));
    let line = Line::from(vec![prompt, input]);

    Paragraph::new(line).block(block)
}

fn build_echo_pane(state: &DebuggerState) -> impl Widget {
    let message = state.last_command_response().unwrap_or("");
    Paragraph::new(Line::from(message))
}

fn build_output_pane(state: &DebuggerState, process: &Process) -> impl Widget {
    let mut header = "output".to_string();
    if let Some(pid) = process.pid() {
        header.push_str(&format!(" - pid: {:?}", pid));
    };
    let block = build_bounding_rect(&DebuggerPane::Logs, Some(header), state);

    // TODO: dynamically adjust to the pane size? Kinda depnds on the width of
    // the lines and if they wrap ... :shrug:
    let log_lines = process.last_n_log_lines(16);
    let text_lines: Vec<Line> = log_lines.iter().map(|line| line.as_str().into()).collect();

    Paragraph::new(text_lines)
        .style(Style::default().fg(Color::White))
        .block(block)
}

fn build_source_pane(state: &DebuggerState) -> impl Widget {
    let block = build_bounding_rect(&DebuggerPane::Source, None, state);
    Paragraph::new("println!(\"hello, world\");")
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
    state: &TuiState,
    _debugger: &Debugger,
    process: &Process,
    frame: &mut Frame,
    rect: Rect,
) {
    let minibuffer_len = if state.debugger_state.last_command_response().is_some() {
        5
    } else {
        3
    };

    let [src, logs, minibuffer] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
            Constraint::Length(minibuffer_len),
        ])
        .areas(rect);

    ///////////////////////////////
    // build top chunk (source and variable panes ...)
    let top_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(src);

    // source pane
    let source_pane = build_source_pane(&state.debugger_state);
    frame.render_widget(source_pane, top_pane_chunks[0]);
    // pane with locals / other ...
    let others_pane = build_watchers_pane(state);
    frame.render_widget(others_pane, top_pane_chunks[1]);

    /////////////////////////////
    // build logs chunk (stdout)
    let bottom_pane_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(logs);
    // logs/stdout pane
    let output_pane = build_output_pane(&state.debugger_state, process);
    frame.render_widget(output_pane, bottom_pane_chunks[0]);

    /////////////////////////////
    // build minbuffer (command and echo area)
    if state.debugger_state.last_command_response().is_some() {
        let minibuffer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(2)])
            .split(minibuffer);
        // command pane
        let command_pane = build_command_pane(&state.debugger_state);
        frame.render_widget(command_pane, minibuffer_chunks[0]);
        // echo pane
        let echo_pane = build_echo_pane(&state.debugger_state);
        frame.render_widget(echo_pane, minibuffer_chunks[1]);
    } else {
        // only render the command line when there is no message to show
        let command_pane = build_command_pane(&state.debugger_state);
        frame.render_widget(command_pane, minibuffer);
    }
}

fn render_logging_screen(state: &DebuggerLogScreenState, frame: &mut Frame, rect: Rect) {
    // this implementation is based on the example in tui-looger:
    // https://github.com/gin66/tui-logger/blob/master/examples/demo.rs
    let [smart_area, main_area, help_area] = Layout::vertical([
        Constraint::Fill(50),
        Constraint::Fill(30),
        Constraint::Length(3),
    ])
    .areas(rect);

    // show two TuiWidgetState side-by-side
    let [left, right] = Layout::horizontal([Constraint::Fill(1); 2]).areas(main_area);

    TuiLoggerSmartWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Cyan))
        .output_separator(':')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(true)
        .output_file(true)
        .output_line(true)
        .state(state.current_state())
        .render(smart_area, frame.buffer_mut());

    // An example of filtering the log output. The left TuiLoggerWidget is filtered to only show
    // log entries for the "App" target. The right TuiLoggerWidget shows all log entries.
    let filter_state = TuiWidgetState::new()
        .set_default_display_level(LevelFilter::Off)
        .set_level_for_target("App", LevelFilter::Debug)
        .set_level_for_target("background-task", LevelFilter::Info);
    let formatter: Option<Box<dyn LogFormatter>> = None;

    TuiLoggerWidget::default()
        .block(Block::bordered().title("Filtered TuiLoggerWidget"))
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style(Style::default().fg(Color::White))
        .state(&filter_state)
        .render(left, frame.buffer_mut());

    TuiLoggerWidget::default()
        .block(Block::bordered().title("Unfiltered TuiLoggerWidget"))
        .opt_formatter(formatter)
        .output_separator('|')
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Long))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .style(Style::default().fg(Color::White))
        .render(right, frame.buffer_mut());

    Text::from(vec![
        "Q: Quit | Tab: Switch state | ↑/↓: Select target | f: Focus target".into(),
        "←/→: Display level | +/-: Filter level | Space: Toggle hidden targets".into(),
        "h: Hide target selector | PageUp/Down: Scroll | Esc: Cancel scroll".into(),
    ])
    .style(Color::Gray)
    .centered()
    .render(help_area, frame.buffer_mut());
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
            render_debugger_screen(state, debugger, process, frame, chunks[1])
        }
        ScreenMode::DebuggerLogging => {
            render_logging_screen(&state.logging_state, frame, chunks[1])
        }
    }
}
