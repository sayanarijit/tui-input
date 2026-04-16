use super::*;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

#[test]
fn handle_tab() {
    let evt = Event::Key(KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });

    let req = to_input_request(&evt);

    assert!(req.is_none());
}

#[test]
fn handle_repeat() {
    let evt = Event::Key(KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Repeat,
        state: KeyEventState::NONE,
    });

    let req = to_input_request(&evt);

    assert_eq!(req, Some(InputRequest::InsertChar('a')));
}
