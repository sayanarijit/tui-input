//! Core logic for handling input.
//!
//! # Units
//!
//! A string has four different possible notions of length or position:
//!
//! - **bytes**:  indices into the UTF-8 encoding, used only internally.
//! - **codepoints**:  Unicode scalar values (what [`str::chars`] yields).
//!   This is what [`Input::cursor`] returns and what
//!   [`InputRequest::SetCursor`] accepts.
//! - **graphemes**:  user-perceived characters (per `unicode-segmentation`).
//!   Movement and deletion ([`InputRequest::GoToPrevChar`],
//!   [`InputRequest::GoToNextChar`], [`InputRequest::DeletePrevChar`],
//!   [`InputRequest::DeleteNextChar`], [`InputRequest::GoToPrevWord`],
//!   [`InputRequest::GoToNextWord`], [`InputRequest::DeletePrevWord`],
//!   [`InputRequest::DeleteNextWord`]) step one *grapheme* or *word*
//!   at a time, which may span multiple codepoints.
//! - **display columns**:  terminal cell width (per `unicode-width`).
//!   Returned by [`Input::visual_cursor`] and [`Input::visual_scroll`].
//!
//! All four can differ for one string.  For example, `🤦🏼‍♂️` is
//! actually `"🤦🏼\u{200D}♂\u{FE0F}"`, which is 17 bytes, 5 codepoints,
//! 1 grapheme, 2 display columns.
//!
//! # Example: Without any backend
//!
//! ```
//! use tui_input::{Input, InputRequest, StateChanged};
//!
//! let mut input: Input = "Hello Worl".into();
//!
//! let req = InputRequest::InsertChar('d');
//! let resp = input.handle(req);
//!
//! assert_eq!(resp, Some(StateChanged { value: true, cursor: true }));
//! assert_eq!(input.cursor(), 11);
//! assert_eq!(input.to_string(), "Hello World");
//! ```

use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

fn prev_grapheme(s: &str, byte: usize) -> Option<usize> {
    GraphemeCursor::new(byte, s.len(), true)
        .prev_boundary(s, 0)
        .ok()
        .flatten()
}

fn next_grapheme(s: &str, byte: usize) -> Option<usize> {
    GraphemeCursor::new(byte, s.len(), true)
        .next_boundary(s, 0)
        .ok()
        .flatten()
}

fn is_word(s: &str) -> bool {
    s.chars()
        .any(|c| !c.is_whitespace() && !c.is_ascii_punctuation())
}

fn prev_word_byte(s: &str, byte: usize) -> usize {
    let mut words = s
        .split_word_bound_indices()
        .filter(|(i, _)| *i < byte)
        .rev();
    while let Some((i, word)) = words.next() {
        if is_word(word) {
            return i;
        }
    }
    0
}

fn next_word_byte(s: &str, byte: usize) -> usize {
    let mut words = s.split_word_bound_indices().filter(|(i, _)| *i > byte);
    while let Some((i, word)) = words.next() {
        if is_word(word) {
            return i;
        }
    }
    s.len()
}

fn codepoint_to_byte(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map_or(s.len(), |(i, _)| i)
}

fn byte_to_codepoint(s: &str, byte: usize) -> usize {
    s[..byte].chars().count()
}

enum Side {
    Left,
    Right,
}

/// Input requests are used to change the input state.
///
/// Different backends can be used to convert events into requests.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputRequest {
    SetCursor(usize),
    InsertChar(char),
    GoToPrevChar,
    GoToNextChar,
    GoToPrevWord,
    GoToNextWord,
    GoToStart,
    GoToEnd,
    DeletePrevChar,
    DeleteNextChar,
    DeletePrevWord,
    DeleteNextWord,
    DeleteLine,
    DeleteTillEnd,
    Yank,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateChanged {
    pub value: bool,
    pub cursor: bool,
}

pub type InputResponse = Option<StateChanged>;

/// The input buffer with cursor support.
///
/// Example:
///
/// ```
/// use tui_input::Input;
///
/// let input: Input = "Hello World".into();
///
/// assert_eq!(input.cursor(), 11);
/// assert_eq!(input.to_string(), "Hello World");
/// ```
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Input {
    value: String,
    /// Codepoints preceding the cursor.  See the module-level `Units` section.
    cursor: usize,
    yank: String,
    last_was_cut: bool,
}

impl Input {
    /// Initialize a new instance with a given value
    /// Cursor will be set to the given value's length.
    pub fn new(value: String) -> Self {
        let len = value.chars().count();
        Self {
            value,
            cursor: len,
            yank: String::new(),
            last_was_cut: false,
        }
    }

    /// Set the value manually.
    /// Cursor will be set to the given value's length.
    pub fn with_value(mut self, value: String) -> Self {
        self.cursor = value.chars().count();
        self.value = value;
        self
    }

    /// Set the cursor manually.
    /// If the input is larger than the value length, it'll be auto adjusted.
    pub fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = cursor.min(self.value.chars().count());
        self
    }

    // Reset the cursor and value to default
    pub fn reset(&mut self) {
        self.cursor = Default::default();
        self.value = Default::default();
    }

    // Reset the cursor and value to default, returning the previous value
    pub fn value_and_reset(&mut self) -> String {
        let val = self.value.clone();
        self.reset();
        val
    }

    fn add_to_yank(&mut self, deleted: String, side: Side) {
        if self.last_was_cut {
            match side {
                Side::Left => self.yank.insert_str(0, &deleted),
                Side::Right => self.yank.push_str(&deleted),
            }
        } else {
            self.yank = deleted;
        }
    }

    fn set_last_was_cut(&mut self, req: InputRequest) {
        use InputRequest::*;
        self.last_was_cut = matches!(
            req,
            DeleteLine | DeletePrevWord | DeleteNextWord | DeleteTillEnd
        );
    }

    /// Handle request and emit response.
    pub fn handle(&mut self, req: InputRequest) -> InputResponse {
        use InputRequest::*;
        let result = match req {
            SetCursor(pos) => {
                let pos = pos.min(self.value.chars().count());
                if self.cursor == pos {
                    None
                } else {
                    self.cursor = pos;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }
            InsertChar(c) => {
                if self.cursor == self.value.chars().count() {
                    self.value.push(c);
                } else {
                    self.value = self
                        .value
                        .chars()
                        .take(self.cursor)
                        .chain(
                            std::iter::once(c)
                                .chain(self.value.chars().skip(self.cursor)),
                        )
                        .collect();
                }
                self.cursor += 1;
                Some(StateChanged {
                    value: true,
                    cursor: true,
                })
            }

            DeletePrevChar => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let prev = prev_grapheme(&self.value, byte)?;
                let removed = self.value[prev..byte].chars().count();
                self.value.replace_range(prev..byte, "");
                self.cursor -= removed;
                Some(StateChanged {
                    value: true,
                    cursor: true,
                })
            }

            DeleteNextChar => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let next = next_grapheme(&self.value, byte)?;
                self.value.replace_range(byte..next, "");
                Some(StateChanged {
                    value: true,
                    cursor: false,
                })
            }

            GoToPrevChar => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let prev = prev_grapheme(&self.value, byte)?;
                self.cursor -= self.value[prev..byte].chars().count();
                Some(StateChanged {
                    value: false,
                    cursor: true,
                })
            }

            GoToPrevWord => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let prev = prev_word_byte(&self.value, byte);
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = byte_to_codepoint(&self.value, prev);
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToNextChar => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let next = next_grapheme(&self.value, byte)?;
                self.cursor += self.value[byte..next].chars().count();
                Some(StateChanged {
                    value: false,
                    cursor: true,
                })
            }

            GoToNextWord => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let next = next_word_byte(&self.value, byte);
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.cursor = byte_to_codepoint(&self.value, next);
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            DeleteLine => {
                if self.value.is_empty() {
                    None
                } else {
                    let side = if self.cursor == self.value.chars().count() {
                        Side::Left
                    } else {
                        Side::Right
                    };
                    self.add_to_yank(self.value.clone(), side);
                    self.value = "".into();
                    self.cursor = 0;
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }

            DeletePrevWord => {
                if self.cursor == 0 {
                    None
                } else {
                    let byte = codepoint_to_byte(&self.value, self.cursor);
                    let prev = prev_word_byte(&self.value, byte);
                    let deleted = self.value[prev..byte].to_string();
                    self.add_to_yank(deleted, Side::Left);
                    self.value.replace_range(prev..byte, "");
                    self.cursor = byte_to_codepoint(&self.value, prev);
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }

            DeleteNextWord => {
                let byte = codepoint_to_byte(&self.value, self.cursor);
                let next = next_word_byte(&self.value, byte);
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    let deleted = self.value[byte..next].to_string();
                    self.add_to_yank(deleted, Side::Right);
                    self.value.replace_range(byte..next, "");
                    Some(StateChanged {
                        value: true,
                        cursor: false,
                    })
                }
            }

            GoToStart => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = 0;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToEnd => {
                let count = self.value.chars().count();
                if self.cursor == count {
                    None
                } else {
                    self.cursor = count;
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            DeleteTillEnd => {
                let deleted: String = self.value.chars().skip(self.cursor).collect();
                self.add_to_yank(deleted, Side::Right);
                self.value = self.value.chars().take(self.cursor).collect();
                Some(StateChanged {
                    value: true,
                    cursor: false,
                })
            }

            Yank => {
                if self.yank.is_empty() {
                    None
                } else if self.cursor == self.value.chars().count() {
                    self.value.push_str(&self.yank);
                    self.cursor += self.yank.chars().count();
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                } else {
                    self.value = self
                        .value
                        .chars()
                        .take(self.cursor)
                        .chain(self.yank.chars())
                        .chain(self.value.chars().skip(self.cursor))
                        .collect();
                    self.cursor += self.yank.chars().count();
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }
        };
        self.set_last_was_cut(req);
        result
    }

    /// Get a reference to the current value.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the number of **codepoints** preceding the cursor.  Movement
    /// and deletion operations step one *grapheme* at a time, so a single
    /// [`InputRequest::GoToNextChar`] or [`InputRequest::DeletePrevChar`]
    /// may change this count by more than one.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the cursor's position in **display columns** (per
    /// `unicode-width`).
    pub fn visual_cursor(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }

        // Safe, because the end index will always be within bounds
        unicode_width::UnicodeWidthStr::width(unsafe {
            self.value.get_unchecked(
                0..self
                    .value
                    .char_indices()
                    .nth(self.cursor)
                    .map_or_else(|| self.value.len(), |(index, _)| index),
            )
        })
    }

    /// Get the scroll position with account for multispace characters.
    pub fn visual_scroll(&self, width: usize) -> usize {
        let scroll = (self.visual_cursor()).max(width) - width;
        let mut uscroll = 0;
        let mut chars = self.value().chars();

        while uscroll < scroll {
            match chars.next() {
                Some(c) => {
                    uscroll += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                }
                None => break,
            }
        }
        uscroll
    }
}

impl From<Input> for String {
    fn from(input: Input) -> Self {
        input.value
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self::new(value.into())
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg(test)]
mod tests {

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
        assert_eq!(input.value()[codepoint_to_byte(&input.value, input.cursor())..].chars().next(), Some('w'));

        input.handle(InputRequest::GoToNextWord);
        // "🤦🏼‍♂️" is now considered a word.
        assert_eq!(
            input.value()[codepoint_to_byte(&input.value, input.cursor())..]
                .chars()
                .next(),
            Some('🤦')
        );

        input.handle(InputRequest::GoToNextWord);
        assert_eq!(
            input.value()[codepoint_to_byte(&input.value, input.cursor())..]
                .chars()
                .next(),
            Some('o')
        );

        // Prev word
        input.handle(InputRequest::GoToPrevWord);
        assert_eq!(
            input.value()[codepoint_to_byte(&input.value, input.cursor())..]
                .chars()
                .next(),
            Some('🤦')
        );

        input.handle(InputRequest::GoToPrevWord);
        assert_eq!(
            input.value()[codepoint_to_byte(&input.value, input.cursor())..]
                .chars()
                .next(),
            Some('w')
        );

        input.handle(InputRequest::GoToPrevWord);
        assert_eq!(input.value()[codepoint_to_byte(&input.value, input.cursor())..].chars().next(), Some('H'));
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
}
