//! This library makes it easy to create finite state machines to tokenize strings.
//!
//! Here's the simplest automaton you can make with it, it simply finds EOF:
//! ```
//! use ram::Automaton;
//!
//! enum TokenType {
//!     End,
//! }
//!
//! // Create the FSM (2 states, 0 or 1) that will parse the source code
//! let mut am = Automaton::new(0, 1);
//! // When the FSM hits the end of the source, go to state 1, the final state
//! am.find_end(TokenType::End as i32, 0, 1);
//!
//! // Run the FSM with an empty string as the source code
//! let source_code = format!("");
//! let runner = am.run(source_code);
//!
//! assert_eq!(runner.tokens.len(), 1);
//! assert!(runner.completed());
//!
//! // With a non-empty string, the result is not complete
//! let source_code = format!("Invalid entry");
//! let runner = am.run(source_code);
//!
//! assert_eq!(runner.tokens.len(), 0);
//! assert!(!runner.completed());
//! ```
//!
//! Run `cargo run --example let-it-be-42` to see a more complete example.

mod ram;

pub use crate::ram::*;
