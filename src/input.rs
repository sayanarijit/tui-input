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

mod value;

use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

use self::value::Value;

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
    let words = s
        .split_word_bound_indices()
        .filter(|(i, _)| *i < byte)
        .rev();
    for (i, word) in words {
        if is_word(word) {
            return i;
        }
    }
    0
}

fn next_word_byte(s: &str, byte: usize) -> usize {
    let words = s.split_word_bound_indices().filter(|(i, _)| *i > byte);
    for (i, word) in words {
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
    value: Value,
    /// Codepoints preceding the cursor.  See the module-level `Units` section.
    cursor: usize,
    yank: String,
    last_was_cut: bool,
}

impl Input {
    /// Initialize a new instance with a given value
    /// Cursor will be set to the given value's length.
    pub fn new(value: String) -> Self {
        let value = Value::new(value);
        let cursor = value.chars();
        Self {
            value,
            cursor,
            yank: String::new(),
            last_was_cut: false,
        }
    }

    /// Set the value manually.
    /// Cursor will be set to the given value's length.
    pub fn with_value(mut self, value: String) -> Self {
        self.value = Value::new(value);
        self.cursor = self.value.chars();
        self
    }

    /// Set the cursor manually.
    /// If the input is larger than the value length, it'll be auto adjusted.
    pub fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = cursor.min(self.value.chars());
        self
    }

    // Reset the cursor and value to default
    pub fn reset(&mut self) {
        self.cursor = Default::default();
        self.value = Default::default();
    }

    // Reset the cursor and value to default, returning the previous value
    pub fn value_and_reset(&mut self) -> String {
        let val = self.value.as_str().to_owned();
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
                let pos = pos.min(self.value.chars());
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
                let byte = codepoint_to_byte(self.value.as_str(), self.cursor);
                self.value.edit().insert(byte, c);
                self.cursor += 1;
                Some(StateChanged {
                    value: true,
                    cursor: true,
                })
            }

            DeletePrevChar => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let prev = prev_grapheme(s, byte)?;
                let removed = s[prev..byte].chars().count();
                self.value.edit().replace_range(prev..byte, "");
                self.cursor -= removed;
                Some(StateChanged {
                    value: true,
                    cursor: true,
                })
            }

            DeleteNextChar => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let next = next_grapheme(s, byte)?;
                self.value.edit().replace_range(byte..next, "");
                Some(StateChanged {
                    value: true,
                    cursor: false,
                })
            }

            GoToPrevChar => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let prev = prev_grapheme(s, byte)?;
                self.cursor -= s[prev..byte].chars().count();
                Some(StateChanged {
                    value: false,
                    cursor: true,
                })
            }

            GoToPrevWord => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let prev = prev_word_byte(s, byte);
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = byte_to_codepoint(s, prev);
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            GoToNextChar => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let next = next_grapheme(s, byte)?;
                self.cursor += s[byte..next].chars().count();
                Some(StateChanged {
                    value: false,
                    cursor: true,
                })
            }

            GoToNextWord => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let next = next_word_byte(s, byte);
                if self.cursor == self.value.chars() {
                    None
                } else {
                    self.cursor = byte_to_codepoint(s, next);
                    Some(StateChanged {
                        value: false,
                        cursor: true,
                    })
                }
            }

            DeleteLine => {
                if self.value.as_str().is_empty() {
                    None
                } else {
                    let side = if self.cursor == self.value.chars() {
                        Side::Left
                    } else {
                        Side::Right
                    };
                    self.add_to_yank(self.value.as_str().to_owned(), side);
                    self.value.edit().clear();
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
                    let s = self.value.as_str();
                    let byte = codepoint_to_byte(s, self.cursor);
                    let prev = prev_word_byte(s, byte);
                    let deleted = s[prev..byte].to_string();
                    self.add_to_yank(deleted, Side::Left);
                    self.value.edit().replace_range(prev..byte, "");
                    self.cursor = byte_to_codepoint(self.value.as_str(), prev);
                    Some(StateChanged {
                        value: true,
                        cursor: true,
                    })
                }
            }

            DeleteNextWord => {
                let s = self.value.as_str();
                let byte = codepoint_to_byte(s, self.cursor);
                let next = next_word_byte(s, byte);
                if self.cursor == self.value.chars() {
                    None
                } else {
                    let deleted = s[byte..next].to_string();
                    self.add_to_yank(deleted, Side::Right);
                    self.value.edit().replace_range(byte..next, "");
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
                let count = self.value.chars();
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
                let byte = codepoint_to_byte(self.value.as_str(), self.cursor);
                let deleted = self.value.as_str()[byte..].to_string();
                self.add_to_yank(deleted, Side::Right);
                self.value.edit().truncate(byte);
                Some(StateChanged {
                    value: true,
                    cursor: false,
                })
            }

            Yank => {
                if self.yank.is_empty() {
                    None
                } else {
                    let byte = codepoint_to_byte(self.value.as_str(), self.cursor);
                    self.value.edit().insert_str(byte, &self.yank);
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

        let s = self.value.as_str();
        // Safe, because the end index will always be within bounds
        unicode_width::UnicodeWidthStr::width(unsafe {
            s.get_unchecked(
                0..s.char_indices()
                    .nth(self.cursor)
                    .map_or_else(|| s.len(), |(index, _)| index),
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
        input.value.into()
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
        self.value.as_str().fmt(f)
    }
}

#[cfg(test)]
mod tests;
