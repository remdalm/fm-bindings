// src/ffi.rs
// FFI (Foreign Function Interface) layer for Swift FoundationModels integration
// This module contains all C-ABI declarations for both blocking and streaming modes

use std::os::raw::{c_char, c_void};

// FFI Type Definitions
// These match the Swift functions exported with @_cdecl
// Callback types with user_data for thread-safe state management
// Must match Swift's @convention(c) signatures exactly

/// Called for each chunk during generation
/// - chunk: null-terminated C string containing the chunk text
/// - user_data: opaque pointer to user state
pub type ChunkCallbackWithData = extern "C" fn(*const c_char, *mut c_void);

/// Called when generation completes successfully
/// - user_data: opaque pointer to user state
pub type DoneCallbackWithData = extern "C" fn(*mut c_void);

/// Called when an error occurs during generation
/// - error: null-terminated C string containing error message
/// - user_data: opaque pointer to user state
pub type ErrorCallbackWithData = extern "C" fn(*const c_char, *mut c_void);

// External Swift Functions
// These functions are implemented in Swift and exported via @_cdecl

#[link(name = "FoundationModelsFFI", kind = "static")]
unsafe extern "C" {
    /// Check if Foundation Model is available on this system
    /// Returns true if the model is available, false otherwise
    ///
    /// This should be called before creating a session to fail-fast
    /// if Apple Intelligence is not enabled or the system is unsupported
    pub fn fm_check_availability() -> bool;

    /// Generate a complete response (blocking mode)
    /// Waits for the entire response before returning via callbacks
    ///
    /// - prompt: null-terminated C string
    /// - user_data: opaque pointer passed to all callbacks
    /// - on_chunk: called for each chunk generated
    /// - on_done: called when generation completes
    /// - on_error: called if error occurs
    pub fn fm_response(
        prompt: *const c_char,
        user_data: *mut c_void,
        on_chunk: ChunkCallbackWithData,
        on_done: DoneCallbackWithData,
        on_error: ErrorCallbackWithData,
    );

    /// Start streaming a Foundation Model response
    /// Returns immediately and delivers chunks via callbacks
    ///
    /// - prompt: null-terminated C string
    /// - user_data: opaque pointer passed to all callbacks
    /// - on_chunk: called for each chunk as it arrives
    /// - on_done: called when stream completes
    /// - on_error: called if error occurs
    pub fn fm_start_stream(
        prompt: *const c_char,
        user_data: *mut c_void,
        on_chunk: ChunkCallbackWithData,
        on_done: DoneCallbackWithData,
        on_error: ErrorCallbackWithData,
    );

    /// Stop/cancel current stream
    pub fn fm_stop_stream();
}
