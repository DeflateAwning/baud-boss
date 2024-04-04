# baud-boss
A feature-rich UART serial terminal, written in Rust

## Getting Started

0. Install [rustup](https://rustup.rs), and then run `rustup update` to install `cargo`.

```bash
cargo install --git https://github.com/DeflateAwning/baud-boss

# run with the command:
baud-boss
```

## Features
* print out incoming serial terminal content
* access sent message history by pressing the up arrow key

## Upcoming Features
[ ] send messages character-by-character
[ ] send messages by preparing them in an input box
[ ] control keys/buttons: pause output, clear output, print a newline right now
[ ] configurable EOL behaviour (TX)
[ ] configurable EOL behaviour (RX)
[ ] UI for selecting options, instead of requiring they be supplied by CLI args
[ ] end-of-message character (optional)
[ ] start-of-message timestamps (with configurable timeouts, maybe)
[ ] installable via `cargo` and crates.io
[ ] show incoming non-printable bytes (or all bytes) as hex
[ ] send hex as raw bytes
[ ] configuration files per-project/workspace (similar to `.vscode/settings.json`), for quickly starting the right EOL, baud, etc. for an embedded systems projects
[ ] log sessions to text file
[ ] log sessions to JSON/YAML/other files
[ ] send a file
[ ] encoding
[ ] filters
[ ] send the `Ctrl+]`, etc. control characters to the remote
[ ] horizontal scrolling of output window
[ ] select from known common baud rates
[ ] scrolling, scroll bar

## Known Issues
* keybinding in footer are still changing; guess-and-check until you bind some that work

## Inspiration
* [pyserial](https://github.com/pyserial/pyserial) (for its ubiquity)
* [ttyper](https://github.com/max-niederman/ttyper) (for its UI)
