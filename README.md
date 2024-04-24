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
* View RX'd (incoming) data.
* Send messages by preparing them in an input box.
* Access sent message history by pressing the up arrow key.

## Upcoming Features
- [ ] installable via `cargo` and crates.io
- [ ] send hex as raw bytes
- [ ] configuration screen
- [ ] keybinding display (maybe click-able)
- [ ] send messages character-by-character
- [ ] control keys/buttons: pause output, clear output, print a newline right now
- [ ] configurable EOL behaviour (TX)
- [ ] configurable EOL behaviour (RX)
- [ ] UI for selecting options, instead of requiring they be supplied by CLI args
- [ ] end-of-message character (optional)
- [ ] start-of-message timestamps (with configurable timeouts, maybe)
- [ ] show incoming non-printable bytes (or all bytes) as hex
- [ ] configuration files per-project/workspace (similar to `.vscode/settings.json`), for quickly starting the right EOL, baud, etc. for an embedded systems projects
- [ ] log sessions to text file
- [ ] log sessions to JSON/YAML/other files
- [ ] send a file
- [ ] encoding
- [ ] filters
- [ ] send the `Ctrl+]`, etc. control characters to the remote
- [ ] select from known common baud rates
- [ ] pre-load a list of commands/messages to send, and pick from the list
- [ ] incoming characters-per-second and lines-per-second counter
- [ ] receive and format incoming ndjson (aka jsonl) data
- [ ] line wrapping configuration in transfer log region

### Minor Features Completed
- [x] scrolling, scroll bar
- [x] horizontal scrolling of transfer log region

### Known Issues
* keybinding in footer are still changing; guess-and-check until you find some that work

## Dependencies/Acknowledgements
* [ratatui](https://github.com/ratatui-org/ratatui)
* [serialport5](https://gitlab.com/susurrus/serialport-rs)

## Inspiration
* [pyserial](https://github.com/pyserial/pyserial) (for its ubiquity)
* [ttyper](https://github.com/max-niederman/ttyper) (for its UI)

## Contributing
* Please Star this repo if it's useful! Share it with your friends.
* Please submit PRs which fix any TODO messages within.
* Please submit PRs which implement any of the features above.
* Please submit Issues with any feature requests and bug reports!

For local dev testing on Linux, use the following to create two connected virtual serial ports:
```bash
socat PTY,link=/dev/ttyS10 PTY,link=/dev/ttyS11

# in a new window (do twice, selecting the two ports from the last step):
cargo build && sudo ./target/debug/baud-boss
```
