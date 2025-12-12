# jdb (jason's debugger)

# Overview
This project is a rust-based, command line debugger. The project only targets linux/x86_64.

Based on Sy Brand's ["Building a Debugger"](https://nostarch.com/building-a-debugger) book, this is my attempt to implement the debugger in rust.

The name is intentionally confusing, overlapping `jdb` with the GOAT debugger `gdb`, the java `jdb` debugger, as well as overlapping with `jdbc`. 
The last is entertaining as I've spent damn near two decades in databass-land, so yet another "db" project (albeit a debugger this time).

I've implemented the key bindings from an emacs user PoV. 

# Executing
`cargo --locked run -- name <full path to executable>`

# Details
The TUI is built with the `ratatui` library. Access the lower-level linux structs and system calls is handled via the `libc` and `nix` crates.

## command entry/editing
Using the ratatui widget [tui-textarea](https://github.com/rhysd/tui-textarea) for text entry. The current use is still very much in the nacent stage (i.e. lots of work to do)

## logging
We log to two places from the debugger:

1. `$XDG_STATE_HOME`/jdb - for standard file logging
2. The alternate screen in the TUI, "Debugger logging". It capture much of the same information as the log file, but displays it within the running debugger. It uses the ratatui widget [tui-logger](https://github.com/gin66/tui-logger), which is super helpful.

## Key bindings (TUI)
| Context | Keys | Action |
| --- | --- | --- |
| Global | `F1` | Switch to main debugger screen |
| Global | `F2` | Switch to debugger logging screen |
| Main screen (normal) | `c` / `e` / `Alt`+`x` | Focus command pane (enter edit mode) |
| Main screen (normal) | `s` / `l` / `o` | Focus source / locals / logs panes |
| Main screen (normal) | `Tab` / `Shift`+`Tab` | Cycle pane focus forward/back |
| Main screen (normal) | `q` | Quit debugger |
| Main screen (edit) | `Enter` | Submit current line as a command |
| Main screen (edit) | `Alt`+`x` | Exit edit mode, focus source pane |
| Logging screen | `q` | Quit debugger (dev escape hatch) |
| Logging screen | `Space`, `+`, `-`, `h`, `f`, `Esc`, arrows, `PageUp`/`PageDown` | Forwarded to `tui-logger` widget for navigation/filtering |
