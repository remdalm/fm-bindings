// FFI (Foreign Function Interface) layer for Swift FoundationModels integration
//
// This module contains all C-ABI declarations for session management,
// blocking and streaming response modes.

use std::os::raw::{c_char, c_void};

/// Opaque pointer to a Swift LanguageModelSession
pub type SessionPtr = *mut c_void;

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
    // =========================================================================
    // Availability
    // =========================================================================

    /// Check if Foundation Model is available on this system
    /// Returns true if the model is available, false otherwise
    ///
    /// This should be called before creating a session to fail-fast
    /// if Apple Intelligence is not enabled or the system is unsupported
    pub fn fm_check_availability() -> bool;

    // =========================================================================
    // Session Lifecycle
    // =========================================================================

    /// Create a new LanguageModelSession with optional instructions
    ///
    /// - instructions: null-terminated C string for system instructions, or NULL for none
    /// - Returns: opaque session pointer, or NULL on failure
    ///
    /// The caller owns the returned session and must call fm_destroy_session to free it.
    pub fn fm_create_session(instructions: *const c_char) -> SessionPtr;

    /// Create a new LanguageModelSession from a serialized transcript
    ///
    /// This restores a previous session state, including instructions and conversation history.
    /// Use this to resume conversations across app launches.
    ///
    /// - transcript_json: null-terminated JSON string from fm_get_transcript_json
    /// - Returns: opaque session pointer, or NULL on failure
    ///
    /// The caller owns the returned session and must call fm_destroy_session to free it.
    pub fn fm_create_session_from_transcript(transcript_json: *const c_char) -> SessionPtr;

    /// Destroy a session and free its resources
    ///
    /// - session: session pointer from fm_create_session or fm_create_session_from_transcript
    ///
    /// After this call, the session pointer is invalid and must not be used.
    pub fn fm_destroy_session(session: SessionPtr);

    // =========================================================================
    // Transcript
    // =========================================================================

    /// Get the current session transcript as JSON
    ///
    /// - session: session pointer
    /// - Returns: null-terminated JSON string, or NULL on failure
    ///
    /// The caller owns the returned string and must call fm_free_string to free it.
    /// The JSON can be passed to fm_create_session_from_transcript to restore the session.
    pub fn fm_get_transcript_json(session: SessionPtr) -> *mut c_char;

    /// Free a string allocated by the Swift side
    ///
    /// - ptr: string pointer from fm_get_transcript_json, or NULL (no-op)
    pub fn fm_free_string(ptr: *mut c_char);

    // =========================================================================
    // Response Generation
    // =========================================================================

    /// Generate a complete response (blocking mode)
    ///
    /// Waits for the entire response before returning via callbacks.
    /// The session transcript is updated with the prompt and response.
    ///
    /// - session: session pointer
    /// - prompt: null-terminated C string
    /// - user_data: opaque pointer passed to all callbacks
    /// - on_chunk: called for each chunk generated (may be called multiple times)
    /// - on_done: called when generation completes successfully
    /// - on_error: called if error occurs (mutually exclusive with on_done)
    pub fn fm_session_response(
        session: SessionPtr,
        prompt: *const c_char,
        user_data: *mut c_void,
        on_chunk: ChunkCallbackWithData,
        on_done: DoneCallbackWithData,
        on_error: ErrorCallbackWithData,
    );

    /// Start streaming a response
    ///
    /// Returns immediately and delivers chunks via callbacks as they are generated.
    /// The session transcript is updated with the prompt and response.
    ///
    /// - session: session pointer
    /// - prompt: null-terminated C string
    /// - user_data: opaque pointer passed to all callbacks
    /// - on_chunk: called for each chunk as it arrives
    /// - on_done: called when stream completes successfully
    /// - on_error: called if error occurs (mutually exclusive with on_done)
    pub fn fm_session_stream(
        session: SessionPtr,
        prompt: *const c_char,
        user_data: *mut c_void,
        on_chunk: ChunkCallbackWithData,
        on_done: DoneCallbackWithData,
        on_error: ErrorCallbackWithData,
    );

    /// Cancel the current streaming response on a session
    ///
    /// - session: session pointer
    ///
    /// Safe to call even if no stream is active.
    /// After cancellation, the stream will complete with tokens received so far.
    pub fn fm_session_cancel_stream(session: SessionPtr);
}
