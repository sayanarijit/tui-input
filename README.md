# tui-input

[![Crate Status](https://img.shields.io/crates/v/tui-input.svg)](https://crates.io/crates/tui-input)
[![Docs Status](https://docs.rs/tui-input/badge.svg)](https://docs.rs/tui-input/)

[![tui-input.gif](https://s10.gifyu.com/images/tui-input.gif)](https://github.com/sayanarijit/tui-input/blob/main/examples/ratatui-input/src/main.rs)

A TUI input library supporting multiple backends.

This crate can be used with [tui-rs](https://github.com/fdehau/tui-rs) and [ratatui](https://github.com/tui-rs-revival/ratatui).

For people using `tui-rs` use version `v0.6.*` for people migrating to `ratatui` use latest version.

## Install

Cargo.toml

```toml
# crossterm
tui-input = "*"

# termion
tui-input = { version = "*", features = ["termion"], default-features = false }
```

## Features

- crossterm (default)
- termion
- serde

## Demo

See [examples](https://github.com/sayanarijit/tui-input/tree/main/examples).

```bash
# Run the example with crossterm as backend.
cargo run --example crossterm_input

# Run the example with termion as backend.
cargo run --example termion_input --features termion

# Run the tui-rs example
(cd ./examples/ratatui-input/ && cargo run)
```

## Used in

- [xplr](https://github.com/sayanarijit/xplr)
