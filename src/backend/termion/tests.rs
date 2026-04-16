use super::*;

#[test]
fn handle_tab() {
    let evt = Event::Key(Key::Char('\t'));

    let req = to_input_request(&evt);

    assert!(req.is_none());
}
