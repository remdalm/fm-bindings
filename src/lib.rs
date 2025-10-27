//! # Foundation Models Bindings for Rust
//!
//! Rust bindings for Apple's [Foundation Models framework](https://developer.apple.com/documentation/foundationmodels),
//! providing access to on-device large language models (LLMs) that power Apple Intelligence.
//!
//! ## Requirements
//!
//! - macOS 26+ or iOS 26+
//! - Apple Intelligence enabled in System Settings
//!
//! ## Features
//!
//! - **Blocking Response**: Get complete responses with `response()`
//! - **Streaming Response**: Get real-time incremental updates with `stream_response()`
//! - Type-safe error handling with `Result<T, Error>`
//! - Zero-copy FFI layer for optimal performance
//!
//! ## Examples
//!
//! ### Blocking Response
//!
//! ```no_run
//! use fm_bindings::LanguageModelSession;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let session = LanguageModelSession::new()?;
//!     let response = session.response("What is Rust?")?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```
//!
//! ### Streaming Response
//!
//! ```no_run
//! use fm_bindings::LanguageModelSession;
//! use std::io::{self, Write};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let session = LanguageModelSession::new()?;
//!
//!     session.stream_response("Tell me a story", |chunk| {
//!         print!("{}", chunk);
//!         let _ = io::stdout().flush();
//!     })?;
//!
//!     println!(); // newline after stream
//!     Ok(())
//! }
//! ```

// Internal modules
mod error;
mod ffi;
mod session;

// Public API exports
pub use error::{Error, Result};
pub use session::LanguageModelSession;
