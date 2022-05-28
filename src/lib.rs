//! TUI input library supporting multiple backends.
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
//!
//! See other examples in the [GitHub repo](https://github.com/sayanarijit/tui-input/tree/main/examples).

mod input;

pub mod backend;
pub use input::{Input, InputRequest, InputResponse, StateChanged};
