# baud-boss
A feature-rich UART serial terminal, written in Rust

## Use It

```bash
cargo install --git https://github.com/DeflateAwning/baud-boss
# TODO: how to run it after installing like this?
```

## Features
* print out incoming serial terminal content

## Upcoming Features
[ ] send messages character-by-character
[ ] send messages by preparing them in an input box
[ ] configurable EOL behaviour (TX)
[ ] configurable EOL behaviour (RX)
[ ] UI for selecting options, instead of requiring they be supplied by CLI args
[ ] end-of-message character (optional)
[ ] start-of-message timestamps (with configurable timeouts, maybe)
[ ] installable via `cargo` and crates.io
[ ] show incoming non-printable bytes (or all bytes) as hex
[ ] send hex as raw bytes

## Inspiration
* [pyserial](https://github.com/pyserial/pyserial) (for its ubiquity)
* [ttyper](https://github.com/max-niederman/ttyper) (for its UI)
