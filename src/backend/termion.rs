#[cfg(feature = "ratatui-termion")]
use ratatui::termion;

use super::layout::layout;
use crate::input::InputRequest;
use crate::Input;
use crate::StateChanged;
use std::io::{Result, Write};
use termion::cursor::Goto;
use termion::event::{Event, Key};
use termion::style::Invert;
use termion::style::NoInvert;

/// Converts termion event into input requests.
pub fn to_input_request(evt: &Event) -> Option<InputRequest> {
    use InputRequest::*;
    match *evt {
        Event::Key(Key::Backspace) | Event::Key(Key::Ctrl('h')) => Some(DeletePrevChar),
        Event::Key(Key::Delete) => Some(DeleteNextChar),
        Event::Key(Key::Left) | Event::Key(Key::Ctrl('b')) => Some(GoToPrevChar),
        Event::Key(Key::Right) | Event::Key(Key::Ctrl('f')) => Some(GoToNextChar),
        // Event::Key(Key::Ctrl(Key::Left)) => Some(GoToPrevWord),
        // Event::Key(Key::Ctrl(Key::Right)) => Some(GoToNextWord),
        Event::Key(Key::Ctrl('u')) => Some(DeleteLine),
        Event::Key(Key::Ctrl('w')) => Some(DeletePrevWord),
        // Event::Key(Key::Ctrl(Key::Delete)) => Some(DeleteNextWord),
        Event::Key(Key::Ctrl('y')) => Some(Yank),
        Event::Key(Key::Ctrl('a')) | Event::Key(Key::Home) => Some(GoToStart),
        Event::Key(Key::Ctrl('e')) | Event::Key(Key::End) => Some(GoToEnd),
        Event::Key(Key::Char('\t')) => None,
        Event::Key(Key::Char(c)) => Some(InsertChar(c)),
        _ => None,
    }
}

/// Renders the input UI at the given position with the given width.
pub fn write<W: Write>(
    stdout: &mut W,
    value: &str,
    cursor: usize,
    (x, y): (u16, u16),
    width: u16,
) -> Result<()> {
    let l = layout(value, cursor, width);
    let pad = l.padding();
    write!(
        stdout,
        "{}{NoInvert}{}{Invert}{}{NoInvert}{}{pad}",
        Goto(x + 1, y + 1),
        l.before,
        l.cursor,
        l.after
    )
}

/// Import this trait to implement `Input::handle_event()` for termion.
pub trait EventHandler {
    /// Handle termion event.
    fn handle_event(&mut self, evt: &Event) -> Option<StateChanged>;
}

impl EventHandler for Input {
    /// Handle termion event.
    fn handle_event(&mut self, evt: &Event) -> Option<StateChanged> {
        to_input_request(evt).and_then(|req| self.handle(req))
    }
}

#[cfg(test)]
mod tests;
