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
//! - **Instructions**: Configure model behavior with system prompts
//! - **Transcript Persistence**: Save and restore session state across app launches
//! - Type-safe error handling with `Result<T, Error>`
//!
//! ## Examples
//!
//! ### Basic Usage
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
//! ### With Instructions
//!
//! ```no_run
//! use fm_bindings::LanguageModelSession;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let session = LanguageModelSession::with_instructions(
//!         "You are a friendly assistant. Keep responses brief and helpful."
//!     )?;
//!     let response = session.response("Hello!")?;
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
//!
//! ### Persisting Sessions
//!
//! ```no_run
//! use fm_bindings::LanguageModelSession;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create and use a session
//!     let session = LanguageModelSession::with_instructions("You are a travel guide.")?;
//!     let _ = session.response("Tell me about Paris")?;
//!     let _ = session.response("What about the food?")?;
//!
//!     // Save the transcript
//!     let transcript = session.transcript_json()?;
//!     std::fs::write("session.json", &transcript)?;
//!
//!     // Later: restore the session
//!     let saved = std::fs::read_to_string("session.json")?;
//!     let restored = LanguageModelSession::from_transcript_json(&saved)?;
//!
//!     // Continue with full context
//!     let response = restored.response("Any restaurant recommendations?")?;
//!     println!("{}", response);
//!     Ok(())
//! }
//! ```
//!
//! ### Error Handling
//!
//! ```no_run
//! use fm_bindings::{LanguageModelSession, Error};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let session = match LanguageModelSession::new() {
//!         Ok(s) => s,
//!         Err(Error::ModelNotAvailable) => {
//!             eprintln!("Apple Intelligence not enabled");
//!             return Ok(());
//!         }
//!         Err(e) => return Err(e.into()),
//!     };
//!
//!     let response = session.response("Hello")?;
//!     println!("{}", response);
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
