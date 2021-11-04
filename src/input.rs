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
    Submit,
    Escape,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateChanged {
    pub value: bool,
    pub cursor: bool,
}

/// Input response is emitted to notify about state changes and other events.
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputResponse {
    /// The value or cursor or both changed.
    StateChanged(StateChanged),

    /// Enter was pressed.
    Submitted,

    /// Esc was pressed.
    Escaped,
}

/// The input buffer with cursor support.
///
/// Example:
///
/// ```
/// use tui_input::Input;
///
/// let value = "Hello World".to_string();
/// let input = Input::default().with_value(value);
///
/// assert_eq!(input.value(), "Hello World");
/// assert_eq!(input.cursor(), 11);
/// ```
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Input {
    value: String,
    cursor: usize,
}

impl Input {
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

    /// Handle request and emit response.
    pub fn handle(&mut self, req: InputRequest) -> Option<InputResponse> {
        use InputRequest::*;
        match req {
            SetCursor(pos) => {
                let pos = pos.min(self.value.chars().count());
                if self.cursor == pos {
                    None
                } else {
                    self.cursor = pos;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }
            InsertChar(c) => {
                if self.cursor == self.value.chars().count() {
                    self.value.push(c);
                } else {
                    self.value.insert(self.cursor, c);
                }
                self.cursor += 1;
                Some(InputResponse::StateChanged(StateChanged {
                    value: true,
                    cursor: true,
                }))
            }

            DeletePrevChar => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                    Some(InputResponse::StateChanged(StateChanged {
                        value: true,
                        cursor: true,
                    }))
                }
            }

            DeleteNextChar => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.value.remove(self.cursor);
                    Some(InputResponse::StateChanged(StateChanged {
                        value: true,
                        cursor: false,
                    }))
                }
            }

            GoToPrevChar => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor -= 1;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            GoToPrevWord => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = self
                        .value
                        .chars()
                        .rev()
                        .skip(
                            self.value.chars().count().max(self.cursor)
                                - self.cursor,
                        )
                        .skip_while(|c| !c.is_alphanumeric())
                        .skip_while(|c| c.is_alphanumeric())
                        .count();
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            GoToNextChar => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.cursor += 1;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            GoToNextWord => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.cursor = self
                        .value
                        .chars()
                        .enumerate()
                        .skip(self.cursor)
                        .skip_while(|(_, c)| c.is_alphanumeric())
                        .find(|(_, c)| c.is_alphanumeric())
                        .map(|(i, _)| i)
                        .unwrap_or_else(|| self.value.chars().count());
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            DeleteLine => {
                if self.value.is_empty() {
                    None
                } else {
                    let cursor = self.cursor;
                    self.value = "".into();
                    self.cursor = 0;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: true,
                        cursor: self.cursor == cursor,
                    }))
                }
            }

            DeletePrevWord => {
                if self.cursor == 0 {
                    None
                } else {
                    let remaining = self.value.chars().skip(self.cursor);
                    let rev = self
                        .value
                        .chars()
                        .rev()
                        .skip(
                            self.value.chars().count().max(self.cursor)
                                - self.cursor,
                        )
                        .skip_while(|c| !c.is_alphanumeric())
                        .skip_while(|c| c.is_alphanumeric())
                        .collect::<Vec<char>>();
                    let rev_len = rev.len();
                    self.value =
                        rev.into_iter().rev().chain(remaining).collect();
                    self.cursor = rev_len;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: true,
                        cursor: true,
                    }))
                }
            }

            DeleteNextWord => {
                if self.cursor == self.value.chars().count() {
                    None
                } else {
                    self.value = self
                        .value
                        .chars()
                        .take(self.cursor)
                        .chain(
                            self.value
                                .chars()
                                .skip(self.cursor)
                                .skip_while(|c| c.is_alphanumeric())
                                .skip_while(|c| !c.is_alphanumeric()),
                        )
                        .collect();

                    Some(InputResponse::StateChanged(StateChanged {
                        value: true,
                        cursor: false,
                    }))
                }
            }

            GoToStart => {
                if self.cursor == 0 {
                    None
                } else {
                    self.cursor = 0;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            GoToEnd => {
                let count = self.value.chars().count();
                if self.cursor == count {
                    None
                } else {
                    self.cursor = count;
                    Some(InputResponse::StateChanged(StateChanged {
                        value: false,
                        cursor: true,
                    }))
                }
            }

            Submit => Some(InputResponse::Submitted),

            Escape => Some(InputResponse::Escaped),
        }
    }

    /// Get a reference to the current value.
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Get the currect cursor placement.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

#[cfg(test)]
mod tests {

    const TEXT: &str = "first second, third.";

    use super::*;

    #[test]
    fn set_cursor() {
        let mut input = Input::default().with_value(TEXT.into());

        let req = InputRequest::SetCursor(3);
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(InputResponse::StateChanged(StateChanged {
                value: false,
                cursor: true
            }))
        );

        assert_eq!(input.value(), "first second, third.");
        assert_eq!(input.cursor(), 3);

        let req = InputRequest::SetCursor(30);
        let resp = input.handle(req);

        assert_eq!(input.cursor(), TEXT.chars().count());
        assert_eq!(
            resp,
            Some(InputResponse::StateChanged(StateChanged {
                value: false,
                cursor: true
            }))
        );

        let req = InputRequest::SetCursor(TEXT.chars().count());
        let resp = input.handle(req);

        assert_eq!(input.cursor(), TEXT.chars().count());
        assert_eq!(resp, None);
    }

    #[test]
    fn insert_char() {
        let mut input = Input::default().with_value(TEXT.into());

        let req = InputRequest::InsertChar('x');
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(InputResponse::StateChanged(StateChanged {
                value: true,
                cursor: true
            }))
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
        let mut input = Input::default().with_value(TEXT.into());

        let req = InputRequest::GoToPrevChar;
        let resp = input.handle(req);

        assert_eq!(
            resp,
            Some(InputResponse::StateChanged(StateChanged {
                value: false,
                cursor: true
            }))
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

    // TODO: test remaining
}
