// src/error.rs
// Error types for Foundation Models bindings

use std::fmt;

/// Errors that can occur when using Foundation Models
#[derive(Debug, Clone)]
pub enum Error {
    /// The Foundation Model is not available on this system
    /// This typically means Apple Intelligence is not enabled
    ModelNotAvailable,

    /// The system returned an error during generation
    GenerationError(String),

    /// Invalid input was provided (e.g., empty prompt)
    InvalidInput(String),

    /// An internal FFI error occurred
    InternalError(String),

    /// A mutex or synchronization primitive was poisoned
    /// This indicates a panic occurred while holding a lock
    PoisonError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ModelNotAvailable => {
                write!(
                    f,
                    "Foundation Model not available. Enable Apple Intelligence in System Settings."
                )
            }
            Error::GenerationError(msg) => {
                write!(f, "Generation error: {}", msg)
            }
            Error::InvalidInput(msg) => {
                write!(f, "Invalid input: {}", msg)
            }
            Error::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
            Error::PoisonError => {
                write!(
                    f,
                    "Synchronization primitive poisoned due to panic while holding lock"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for Foundation Models operations
pub type Result<T> = std::result::Result<T, Error>;
