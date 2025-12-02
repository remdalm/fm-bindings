// Language Model Session - the main API for Foundation Models

use crate::error::{Error, Result};
use crate::ffi;
use std::ffi::CString;
use std::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex};

/// A session for interacting with Apple's Foundation Models
///
/// This provides access to on-device language models via the FoundationModels framework.
/// Requires macOS 26+ or iOS 26+ with Apple Intelligence enabled.
///
/// # Session State
///
/// Each session maintains a transcript of all interactions (prompts, responses, etc.).
/// The transcript can be serialized to JSON for persistence and used to restore
/// sessions across app launches.
///
/// # Creating Sessions
///
/// - [`LanguageModelSession::new()`] - Create without instructions
/// - [`LanguageModelSession::with_instructions()`] - Create with system prompt
/// - [`LanguageModelSession::from_transcript_json()`] - Restore from saved transcript
///
/// # Getting Responses
///
/// - [`response()`](Self::response) - Blocking response (waits for completion)
/// - [`stream_response()`](Self::stream_response) - Streaming response (real-time chunks)
/// - [`cancel_stream()`](Self::cancel_stream) - Cancel ongoing stream
///
/// # Examples
///
/// See the method-level documentation for detailed examples:
/// - [`new()`](Self::new) and [`response()`](Self::response) for basic usage
/// - [`stream_response()`](Self::stream_response) for streaming
/// - [`transcript_json()`](Self::transcript_json) and [`from_transcript_json()`](Self::from_transcript_json) for persistence
pub struct LanguageModelSession {
    ptr: NonNull<std::ffi::c_void>,
}

// Safety: The Swift LanguageModelSession is thread-safe (@unchecked Sendable)
unsafe impl Send for LanguageModelSession {}
unsafe impl Sync for LanguageModelSession {}

impl LanguageModelSession {
    /// Creates a new language model session without instructions
    ///
    /// This is equivalent to calling `with_instructions(None)`.
    ///
    /// # Errors
    ///
    /// Returns `Error::ModelNotAvailable` if Apple Intelligence is not enabled
    /// or the system model is unavailable.
    pub fn new() -> Result<Self> {
        Self::with_instructions_opt(None)
    }

    /// Creates a new language model session with instructions
    ///
    /// Instructions define the model's persona, behavior, and guidelines for the
    /// entire session. They are always the first entry in the session transcript.
    ///
    /// # Arguments
    ///
    /// * `instructions` - System prompt that guides the model's behavior
    ///
    /// # Errors
    ///
    /// * `Error::ModelNotAvailable` - If Apple Intelligence is not enabled
    /// * `Error::InvalidInput` - If instructions contain a null byte
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// let session = LanguageModelSession::with_instructions(
    ///     "You are a helpful coding assistant. Provide concise answers."
    /// )?;
    /// # Ok::<(), fm_bindings::Error>(())
    /// ```
    pub fn with_instructions(instructions: &str) -> Result<Self> {
        Self::with_instructions_opt(Some(instructions))
    }

    /// Creates a new language model session with optional instructions
    ///
    /// # Arguments
    ///
    /// * `instructions` - Optional system prompt, or `None` for no instructions
    ///
    /// # Errors
    ///
    /// * `Error::ModelNotAvailable` - If Apple Intelligence is not enabled
    /// * `Error::InvalidInput` - If instructions contain a null byte
    fn with_instructions_opt(instructions: Option<&str>) -> Result<Self> {
        if !unsafe { ffi::fm_check_availability() } {
            return Err(Error::ModelNotAvailable);
        }

        let c_instructions = match instructions {
            Some(s) => Some(
                CString::new(s)
                    .map_err(|_| Error::InvalidInput("Instructions contain null byte".into()))?,
            ),
            None => None,
        };

        let ptr = unsafe {
            ffi::fm_create_session(
                c_instructions
                    .as_ref()
                    .map_or(std::ptr::null(), |s| s.as_ptr()),
            )
        };

        NonNull::new(ptr)
            .map(|ptr| Self { ptr })
            .ok_or_else(|| Error::InternalError("Failed to create session".into()))
    }

    /// Creates a session from a serialized transcript JSON
    ///
    /// This restores a previous session state, including the original instructions
    /// and full conversation history. Use this to resume conversations across
    /// app launches.
    ///
    /// # Arguments
    ///
    /// * `transcript_json` - JSON string from `transcript_json()`
    ///
    /// # Errors
    ///
    /// * `Error::ModelNotAvailable` - If Apple Intelligence is not enabled
    /// * `Error::InvalidInput` - If JSON contains a null byte or is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// let json = std::fs::read_to_string("session.json")?;
    /// let session = LanguageModelSession::from_transcript_json(&json)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_transcript_json(transcript_json: &str) -> Result<Self> {
        if !unsafe { ffi::fm_check_availability() } {
            return Err(Error::ModelNotAvailable);
        }

        let c_json = CString::new(transcript_json)
            .map_err(|_| Error::InvalidInput("Transcript JSON contains null byte".into()))?;

        let ptr = unsafe { ffi::fm_create_session_from_transcript(c_json.as_ptr()) };

        NonNull::new(ptr)
            .map(|ptr| Self { ptr })
            .ok_or_else(|| Error::InternalError("Failed to restore session from transcript".into()))
    }

    /// Gets the current session transcript as JSON
    ///
    /// The returned JSON can be persisted and later passed to `from_transcript_json()`
    /// to restore the session state.
    ///
    /// # Returns
    ///
    /// JSON string representing the full transcript (instructions, prompts, responses)
    ///
    /// # Errors
    ///
    /// * `Error::InternalError` - If transcript serialization fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// let session = LanguageModelSession::new()?;
    /// let _ = session.response("Hello")?;
    ///
    /// let json = session.transcript_json()?;
    /// std::fs::write("session.json", &json)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn transcript_json(&self) -> Result<String> {
        let json_ptr = unsafe { ffi::fm_get_transcript_json(self.ptr.as_ptr()) };

        if json_ptr.is_null() {
            // Empty transcript is valid
            return Ok("[]".to_string());
        }

        let json = unsafe {
            let s = std::ffi::CStr::from_ptr(json_ptr)
                .to_string_lossy()
                .into_owned();
            ffi::fm_free_string(json_ptr);
            s
        };

        Ok(json)
    }

    /// Generates a complete response to the given prompt
    ///
    /// This method blocks until the entire response is generated and returned as a String.
    /// The prompt and response are added to the session transcript.
    ///
    /// For a better user experience with incremental updates, use `stream_response` instead.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input text to send to the model
    ///
    /// # Errors
    ///
    /// * `Error::ModelNotAvailable` - If the Foundation Model is not available
    /// * `Error::InvalidInput` - If the prompt is empty or invalid
    /// * `Error::GenerationError` - If an error occurs during generation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let session = LanguageModelSession::new()?;
    /// let response = session.response("Explain Rust ownership")?;
    /// println!("Response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub fn response(&self, prompt: &str) -> Result<String> {
        if prompt.is_empty() {
            return Err(Error::InvalidInput("Prompt cannot be empty".into()));
        }

        // Create C string for FFI
        let c_prompt = CString::new(prompt)
            .map_err(|_| Error::InvalidInput("Prompt contains null byte".into()))?;

        // Shared state for collecting response
        let state = Arc::new((Mutex::new(ResponseState::default()), Condvar::new()));
        let state_clone = Arc::clone(&state);

        // Call Swift FFI with blocking response mode
        unsafe {
            ffi::fm_response(
                c_prompt.as_ptr(),
                Box::into_raw(Box::new(state_clone)) as *mut _,
                response_callback,
                response_done_callback,
                response_error_callback,
            );
        }

        // Wait for completion
        let (mutex, cvar) = &*state;
        let mut response_state = mutex.lock().map_err(|_| Error::PoisonError)?;
        while !response_state.finished {
            response_state = cvar.wait(response_state).map_err(|_| Error::PoisonError)?;
        }

        // Check for errors
        if let Some(error) = &response_state.error {
            if error.contains("not available") {
                return Err(Error::ModelNotAvailable);
            }
            return Err(Error::GenerationError(error.clone()));
        }

        Ok(response_state.text.clone())
    }

    /// Generates a streaming response to the given prompt
    ///
    /// This method calls the provided callback for each chunk as it's generated,
    /// providing immediate feedback to the user. The callback receives string slices
    /// containing incremental text deltas.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input text to send to the model
    /// * `on_chunk` - Callback function called for each generated chunk
    ///
    /// # Errors
    ///
    /// * `Error::ModelNotAvailable` - If the Foundation Model is not available
    /// * `Error::InvalidInput` - If the prompt is empty or invalid
    /// * `Error::GenerationError` - If an error occurs during generation
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// # use std::io::{self, Write};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let session = LanguageModelSession::new()?;
    ///
    /// session.stream_response("Tell me a story", |chunk| {
    ///     print!("{}", chunk);
    ///     let _ = io::stdout().flush();
    /// })?;
    ///
    /// println!(); // newline after stream completes
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream_response<F>(&self, prompt: &str, on_chunk: F) -> Result<()>
    where
        F: FnMut(&str),
    {
        if prompt.is_empty() {
            return Err(Error::InvalidInput("Prompt cannot be empty".into()));
        }

        // Create C string for FFI
        let c_prompt = CString::new(prompt)
            .map_err(|_| Error::InvalidInput("Prompt contains null byte".into()))?;

        // Shared state for streaming
        let state = Arc::new((Mutex::new(StreamState::default()), Condvar::new()));
        let state_clone = Arc::clone(&state);

        // Call Swift FFI with streaming mode
        unsafe {
            ffi::fm_start_stream(
                c_prompt.as_ptr(),
                Box::into_raw(Box::new((
                    state_clone,
                    Box::new(on_chunk) as Box<dyn FnMut(&str)>,
                ))) as *mut _,
                stream_chunk_callback,
                stream_done_callback,
                stream_error_callback,
            );
        }

        // Wait for completion
        let (mutex, cvar) = &*state;
        let mut stream_state = mutex.lock().map_err(|_| Error::PoisonError)?;
        while !stream_state.finished {
            stream_state = cvar.wait(stream_state).map_err(|_| Error::PoisonError)?;
        }

        // Check for errors
        if let Some(error) = &stream_state.error {
            if error.contains("not available") {
                return Err(Error::ModelNotAvailable);
            }
            return Err(Error::GenerationError(error.clone()));
        }

        Ok(())
    }

    /// Cancels the current streaming response
    ///
    /// This method immediately cancels any ongoing streaming operation started with
    /// `stream_response`. The streaming callback will stop receiving tokens and the
    /// stream will complete with the tokens received so far.
    ///
    /// # Notes
    ///
    /// * This is a global operation that cancels the current stream
    /// * Safe to call even if no stream is active
    /// * After cancellation, the `stream_response` method will return normally
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fm_bindings::LanguageModelSession;
    /// # use std::thread;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let session = LanguageModelSession::new()?;
    /// let session_clone = session.clone();
    ///
    /// // Start streaming in a thread
    /// thread::spawn(move || {
    ///     session_clone.stream_response("Long prompt...", |chunk| {
    ///         print!("{}", chunk);
    ///     }).ok();
    /// });
    ///
    /// // Cancel after a delay
    /// thread::sleep(Duration::from_secs(2));
    /// session.cancel_stream();
    /// # Ok(())
    /// # }
    /// ```
    pub fn cancel_stream(&self) {
        unsafe {
            ffi::fm_stop_stream();
        }
    }
}

// Internal State Types

#[derive(Default)]
struct ResponseState {
    text: String,
    finished: bool,
    error: Option<String>,
}

#[derive(Default)]
struct StreamState {
    finished: bool,
    error: Option<String>,
}

// =============================================================================
// C Callbacks for response()
// =============================================================================

extern "C" fn response_chunk_callback(
    chunk: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    if chunk.is_null() || user_data.is_null() {
        return;
    }

    unsafe {
        let state = &*(user_data as *const Arc<(Mutex<ResponseState>, Condvar)>);
        let chunk_str = std::ffi::CStr::from_ptr(chunk).to_string_lossy();

        let (mutex, _) = &**state;
        if let Ok(mut response_state) = mutex.lock() {
            response_state.text.push_str(&chunk_str);
        }
    }
}

extern "C" fn response_done_callback(user_data: *mut std::os::raw::c_void) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        // Take ownership back from the raw pointer
        let state = Box::from_raw(user_data as *mut Arc<(Mutex<ResponseState>, Condvar)>);

        let (mutex, cvar) = &**state;
        if let Ok(mut response_state) = mutex.lock() {
            response_state.finished = true;
            cvar.notify_all();
        }
    }
}

extern "C" fn response_error_callback(
    error: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        // Take ownership back from the raw pointer
        let state = Box::from_raw(user_data as *mut Arc<(Mutex<ResponseState>, Condvar)>);

        let (mutex, cvar) = &**state;
        if let Ok(mut response_state) = mutex.lock() {
            if !error.is_null() {
                let error_str = std::ffi::CStr::from_ptr(error)
                    .to_string_lossy()
                    .into_owned();
                response_state.error = Some(error_str);
            }
            response_state.finished = true;
            cvar.notify_all();
        }
    }
}

// =============================================================================
// C Callbacks for stream_response()
// =============================================================================

type StreamCallback = Box<dyn FnMut(&str)>;
type StreamUserData = (Arc<(Mutex<StreamState>, Condvar)>, StreamCallback);

extern "C" fn stream_chunk_callback(
    chunk: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    if chunk.is_null() || user_data.is_null() {
        return;
    }

    unsafe {
        let data = &mut *(user_data as *mut StreamUserData);
        let chunk_str = std::ffi::CStr::from_ptr(chunk).to_string_lossy();
        (data.1)(&chunk_str);
    }
}

extern "C" fn stream_done_callback(user_data: *mut std::os::raw::c_void) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        // Take ownership back from the raw pointer
        let data = Box::from_raw(user_data as *mut StreamUserData);

        let (mutex, cvar) = &*data.0;
        if let Ok(mut stream_state) = mutex.lock() {
            stream_state.finished = true;
            cvar.notify_all();
        }
    }
}

extern "C" fn stream_error_callback(
    error: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        // Take ownership back from the raw pointer
        let data = Box::from_raw(user_data as *mut StreamUserData);

        let (mutex, cvar) = &*data.0;
        if let Ok(mut stream_state) = mutex.lock() {
            if !error.is_null() {
                let error_str = std::ffi::CStr::from_ptr(error)
                    .to_string_lossy()
                    .into_owned();
                stream_state.error = Some(error_str);
            }
            stream_state.finished = true;
            cvar.notify_all();
        }
    }
}
