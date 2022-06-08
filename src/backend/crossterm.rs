use crate::InputRequest;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute as CAttribute, Print, SetAttribute},
    Result,
};
use std::io::Write;

/// Converts crossterm event into input requests.
pub fn to_input_request(evt: CrosstermEvent) -> Option<InputRequest> {
    use InputRequest::*;
    use KeyCode::*;
    match evt {
        CrosstermEvent::Key(KeyEvent { code, modifiers }) => match (code, modifiers) {
            (Backspace, KeyModifiers::NONE) => Some(DeletePrevChar),
            (Delete, KeyModifiers::NONE) => Some(DeleteNextChar),
            (Tab, KeyModifiers::NONE) => None,
            (Left, KeyModifiers::NONE) => Some(GoToPrevChar),
            (Left, KeyModifiers::CONTROL) => Some(GoToPrevWord),
            (Right, KeyModifiers::NONE) => Some(GoToNextChar),
            (Right, KeyModifiers::CONTROL) => Some(GoToNextWord),
            (Char('u'), KeyModifiers::CONTROL) => Some(DeleteLine),
            (Char('w'), KeyModifiers::CONTROL) => Some(DeletePrevWord),
            (Delete, KeyModifiers::CONTROL) => Some(DeleteNextWord),
            (Char('a'), KeyModifiers::CONTROL) => Some(GoToStart),
            (Char('e'), KeyModifiers::CONTROL) => Some(GoToEnd),
            (Char(c), KeyModifiers::NONE) => Some(InsertChar(c)),
            (Char(c), KeyModifiers::SHIFT) => Some(InsertChar(c)),
            (_, _) => None,
        },
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
    queue!(stdout, MoveTo(x, y), SetAttribute(CAttribute::NoReverse))?;

    let val_width = width.max(1) as usize - 1;
    let len = value.chars().count();
    let start = (len.max(val_width) - val_width).min(cursor);
    let mut chars = value.chars().skip(start);
    let mut i = start;

    // Chars before cursor
    while i < cursor {
        i += 1;
        let c = chars.next().unwrap_or(' ');
        queue!(stdout, Print(c))?;
    }

    // Cursor
    i += 1;
    let c = chars.next().unwrap_or(' ');
    queue!(
        stdout,
        SetAttribute(CAttribute::Reverse),
        Print(c),
        SetAttribute(CAttribute::NoReverse)
    )?;

    // Chars after the cursor
    while i <= start + val_width {
        i += 1;
        let c = chars.next().unwrap_or(' ');
        queue!(stdout, Print(c))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_tab() {
        let evt = CrosstermEvent::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
        });

        let req = to_input_request(evt);

        assert!(req.is_none());
    }
}
