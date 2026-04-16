const TEXT: &str = "first second, third.";

use super::*;

#[test]
fn format() {
    let input: Input = TEXT.into();
    println!("{}", input);
    println!("{}", input);
}

#[test]
fn set_cursor() {
    let mut input: Input = TEXT.into();

    let req = InputRequest::SetCursor(3);
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: false,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.cursor(), 3);

    let req = InputRequest::SetCursor(30);
    let resp = input.handle(req);

    assert_eq!(input.cursor(), TEXT.chars().count());
    assert_eq!(
        resp,
        Some(StateChanged {
            value: false,
            cursor: true,
        })
    );

    let req = InputRequest::SetCursor(TEXT.chars().count());
    let resp = input.handle(req);

    assert_eq!(input.cursor(), TEXT.chars().count());
    assert_eq!(resp, None);
}

#[test]
fn insert_char() {
    let mut input: Input = TEXT.into();

    let req = InputRequest::InsertChar('x');
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: true,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "first second, third.x");
    assert_eq!(input.cursor(), TEXT.chars().count() + 1);
    input.handle(req);
    assert_eq!(input.value(), "first second, third.xx");
    assert_eq!(input.cursor(), TEXT.chars().count() + 2);

    let mut input = input.with_cursor(3);
    input.handle(req);
    assert_eq!(input.value(), "firxst second, third.xx");
    assert_eq!(input.cursor(), 4);

    input.handle(req);
    assert_eq!(input.value(), "firxxst second, third.xx");
    assert_eq!(input.cursor(), 5);
}

#[test]
fn go_to_prev_char() {
    let mut input: Input = TEXT.into();

    let req = InputRequest::GoToPrevChar;
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: false,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.cursor(), TEXT.chars().count() - 1);

    let mut input = input.with_cursor(3);
    input.handle(req);
    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.cursor(), 2);

    input.handle(req);
    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.cursor(), 1);
}

#[test]
fn remove_unicode_chars() {
    let mut input: Input = "¡test¡".into();

    let req = InputRequest::DeletePrevChar;
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: true,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "¡test");
    assert_eq!(input.cursor(), 5);

    input.handle(InputRequest::GoToStart);

    let req = InputRequest::DeleteNextChar;
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: true,
            cursor: false,
        })
    );

    assert_eq!(input.value(), "test");
    assert_eq!(input.cursor(), 0);
}

#[test]
fn insert_unicode_chars() {
    let mut input = Input::from("¡test¡").with_cursor(5);

    let req = InputRequest::InsertChar('☆');
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: true,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "¡test☆¡");
    assert_eq!(input.cursor(), 6);

    input.handle(InputRequest::GoToStart);
    input.handle(InputRequest::GoToNextChar);

    let req = InputRequest::InsertChar('☆');
    let resp = input.handle(req);

    assert_eq!(
        resp,
        Some(StateChanged {
            value: true,
            cursor: true,
        })
    );

    assert_eq!(input.value(), "¡☆test☆¡");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn multispace_characters() {
    let input: Input = "Ｈｅｌｌｏ, ｗｏｒｌｄ!".into();
    assert_eq!(input.cursor(), 13);
    assert_eq!(input.visual_cursor(), 23);
    assert_eq!(input.visual_scroll(6), 18);
}

#[test]
fn yank_delete_line() {
    let mut input: Input = TEXT.into();
    input.handle(InputRequest::DeleteLine);
    assert_eq!(input.value(), "");
    assert_eq!(input.cursor(), 0);
    assert_eq!(input.yank, TEXT);

    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), TEXT);
    assert_eq!(input.cursor(), TEXT.chars().count());
    assert_eq!(input.yank, TEXT);
}

#[test]
fn yank_delete_till_end() {
    let mut input = Input::from(TEXT).with_cursor(6);
    input.handle(InputRequest::DeleteTillEnd);
    assert_eq!(input.value(), "first ");
    assert_eq!(input.cursor(), 6);
    assert_eq!(input.yank, "second, third.");

    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.cursor(), TEXT.chars().count());
    assert_eq!(input.yank, "second, third.");
}

#[test]
fn yank_delete_prev_word() {
    let mut input = Input::from(TEXT).with_cursor(12);
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.value(), "first , third.");
    assert_eq!(input.yank, "second");

    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.yank, "second");
}

#[test]
fn yank_delete_next_word() {
    let mut input = Input::from(TEXT).with_cursor(6);
    input.handle(InputRequest::DeleteNextWord);
    assert_eq!(input.value(), "first third.");
    assert_eq!(input.yank, "second, ");

    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
    assert_eq!(input.yank, "second, ");
}

#[test]
fn yank_empty() {
    let mut input: Input = TEXT.into();
    let result = input.handle(InputRequest::Yank);
    assert_eq!(result, None);
    assert_eq!(input.value(), TEXT);
    assert_eq!(input.yank, "");
}

#[test]
fn yank_at_middle() {
    let mut input = Input::from(TEXT).with_cursor(6);
    input.handle(InputRequest::DeleteTillEnd);
    assert_eq!(input.value(), "first ");
    assert_eq!(input.yank, "second, third.");
    input.handle(InputRequest::GoToStart);
    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "second, third.first ");
    assert_eq!(input.cursor(), 14);
    assert_eq!(input.yank, "second, third.");
}

#[test]
fn yank_consecutive_delete_prev_word() {
    let mut input = Input::from(TEXT).with_cursor(TEXT.chars().count());
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.value(), "first second, ");
    assert_eq!(input.yank, "third.");
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.value(), "first ");
    assert_eq!(input.yank, "second, third.");
    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
}

#[test]
fn yank_consecutive_delete_next_word() {
    let mut input = Input::from(TEXT).with_cursor(0);
    input.handle(InputRequest::DeleteNextWord);
    assert_eq!(input.value(), "second, third.");
    assert_eq!(input.yank, "first ");
    input.handle(InputRequest::DeleteNextWord);
    assert_eq!(input.value(), "third.");
    assert_eq!(input.yank, "first second, ");
    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
}

#[test]
fn yank_insert_breaks_cut_sequence() {
    let mut input = Input::from(TEXT).with_cursor(TEXT.chars().count());
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.yank, "third.");
    input.handle(InputRequest::InsertChar('x'));
    input.handle(InputRequest::DeletePrevChar);
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.yank, "second, ");
}

#[test]
fn yank_mixed_delete_word_and_line() {
    let mut input = Input::from(TEXT).with_cursor(6);
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.value(), "second, third.");
    assert_eq!(input.yank, "first ");
    input.handle(InputRequest::DeleteLine);
    assert_eq!(input.value(), "");
    assert_eq!(input.yank, "first second, third.");
    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
}

#[test]
fn yank_mixed_delete_word_and_line_from_end() {
    let mut input = Input::from(TEXT).with_cursor(TEXT.chars().count());
    input.handle(InputRequest::DeletePrevWord);
    assert_eq!(input.value(), "first second, ");
    assert_eq!(input.yank, "third.");
    input.handle(InputRequest::DeleteLine);
    assert_eq!(input.value(), "");
    assert_eq!(input.yank, "first second, third.");
    input.handle(InputRequest::Yank);
    assert_eq!(input.value(), "first second, third.");
}

fn walk_grapheme(value: &str, positions: &[usize]) {
    let end = *positions.last().unwrap();

    let mut input: Input = value.into();
    assert_eq!(input.cursor(), end);
    for &pos in positions.iter().rev().skip(1) {
        input.handle(InputRequest::GoToPrevChar);
        assert_eq!(input.cursor(), pos);
    }
    for &pos in &positions[1..] {
        input.handle(InputRequest::GoToNextChar);
        assert_eq!(input.cursor(), pos);
    }

    for &pos in positions.iter().rev().skip(1) {
        input.handle(InputRequest::DeletePrevChar);
        assert_eq!(input.cursor(), pos);
    }
    assert_eq!(input.value(), "");

    let mut input: Input = value.into();
    input.handle(InputRequest::GoToStart);
    for _ in 0..positions.len() - 1 {
        input.handle(InputRequest::DeleteNextChar);
        assert_eq!(input.cursor(), 0);
    }
    assert_eq!(input.value(), "");
}

#[test]
fn grapheme_combining_mark() {
    // á = a + U+0301 = 1 grapheme = 2 codepoints.
    //
    // A letter with a combining accent should be treated the same as
    // if it had been typed in composed form.
    walk_grapheme("xa\u{0301}y", &[0, 1, 3, 4]);
}

#[test]
fn grapheme_facepalm_emoji() {
    // 🤦🏼‍♂️ = 1 grapheme = 5 codepoints.
    //
    // This complex emoji is composed of:
    //
    // FACE PALM
    // EMOJI MODIFIER FITZPATRICK TYPE-3 (aka skin tone)
    // ZERO WIDTH JOINER (combine this emoji with next emoji(!))
    // MALE SIGN
    // VARIATION SELECTOR-16 (interpret previous codepoint as emoji)
    walk_grapheme("x🤦🏼\u{200D}♂\u{FE0F}y", &[0, 1, 6, 7]);
}

#[test]
fn grapheme_flag_sequence() {
    // 🇺🇸 = 1 flag = 2 regional indicators = 1 grapheme = 2 codepoints.
    //
    // Flags are represented by two codepoints from a special range of
    // 26 codepoints, one for each letter A-Z.  A flag is specified by
    // writing the ISO country code using those special codepoints.
    walk_grapheme("x🇺🇸y", &[0, 1, 3, 4]);
}

#[test]
fn word_movement_comprehensive() {
    let mut input: Input = "Hello, world! 🤦🏼‍♂️ ok".into();
    input.handle(InputRequest::GoToStart);

    // Next word
    input.handle(InputRequest::GoToNextWord);
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('w')
    );

    input.handle(InputRequest::GoToNextWord);
    // "🤦🏼‍♂️" is now considered a word.
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('🤦')
    );

    input.handle(InputRequest::GoToNextWord);
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('o')
    );

    // Prev word
    input.handle(InputRequest::GoToPrevWord);
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('🤦')
    );

    input.handle(InputRequest::GoToPrevWord);
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('w')
    );

    input.handle(InputRequest::GoToPrevWord);
    assert_eq!(
        input.value()[codepoint_to_byte(input.value(), input.cursor())..]
            .chars()
            .next(),
        Some('H')
    );
}

#[test]
fn delete_emoji_word() {
    let mut input: Input = "abc 🤦🏼‍♂️ def".into();
    input.handle(InputRequest::GoToEnd);
    input.handle(InputRequest::DeletePrevWord); // deletes "def"
    assert_eq!(input.value(), "abc 🤦🏼‍♂️ ");
    input.handle(InputRequest::DeletePrevWord); // deletes "🤦🏼‍♂️ "
    assert_eq!(input.value(), "abc ");
    input.handle(InputRequest::DeletePrevWord); // deletes "abc "
    assert_eq!(input.value(), "");
}

#[test]
fn delete_word_comprehensive() {
    let mut input: Input = "abc  def, ghi".into();
    input.handle(InputRequest::GoToStart);
    input.handle(InputRequest::GoToNextWord); // at 'd'
    input.handle(InputRequest::DeleteNextWord); // deletes "def, "
    assert_eq!(input.value(), "abc  ghi");
    assert_eq!(input.cursor(), 5);

    input.handle(InputRequest::DeletePrevWord); // deletes "abc  "
    assert_eq!(input.value(), "ghi");
    assert_eq!(input.cursor(), 0);
}
