//! TUI input library supporting multiple backends.
//!
//! # Example: Without any backend
//!
//! ```
//! use tui_input::{Input, InputRequest, InputResponse};
//!
//! let req = InputRequest::InsertChar('x');
//! let mut input = Input::default();
//! let resp = input.handle(req);
//!
//! assert_eq!(resp, InputResponse::StateChanged { value: true, cursor: true });
//! assert_eq!(input.value(), "x");
//! assert_eq!(input.cursor(), 1);
//! ```
//!
//! See other examples on GitHub repository.

mod input;

pub mod backend;
pub use input::{Input, InputRequest, InputResponse};
