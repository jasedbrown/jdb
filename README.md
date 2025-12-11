# jdb (jason's debugger)

# Overview
This project is a rust-based, command line debugger. The project only targets linux/x86_64.

Based on Sy Brand's ["Building a Debugger"](https://nostarch.com/building-a-debugger) book, this is my attempt to implement the debugger in rust.

The name is intentionally confusing, overlapping `jdb` with the GOAT debugger `gdb`, the java `jdb` debugger, as well as overlapping with `jdbc`. 
The last is entertaining as I've spent damn near two decades in databass-land, so yet another "db" project (albeit a debugger this time).

# Executing
`cargo --locked run -- name <full path to executable>`

# Details
The TUI is built with the `ratatui` library. Access the lower-level linux structs and system calls is handled via the `libc` and `nix` crates.
