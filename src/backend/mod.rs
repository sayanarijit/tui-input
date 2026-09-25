#[cfg(any(feature = "ratatui-crossterm", feature = "crossterm"))]
pub mod crossterm;

#[cfg(any(feature = "ratatui-termion", feature = "termion"))]
pub mod termion;

#[cfg(any(
    feature = "ratatui-crossterm",
    feature = "crossterm",
    feature = "ratatui-termion",
    feature = "termion"
))]
mod layout;
