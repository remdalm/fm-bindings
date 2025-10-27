// src/session.rs
// Language Model Session - the main API for Foundation Models

use super::error::{Error, Result};
use super::ffi;
use std::ffi::CString;
use std::sync::{Arc, Condvar, Mutex};

/// A session for interacting with Apple's Foundation Models
///
/// This provides access to on-device language models via the FoundationModels framework.
/// Requires macOS 26+ or iOS 26+ with Apple Intelligence enabled.
///
/// # Examples
///
/// ## Blocking response
/// ```no_run
/// # use fm_bindings::LanguageModelSession;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let session = LanguageModelSession::new()?;
/// let response = session.response("What is Rust?")?;
/// println!("{}", response);
/// # Ok(())
/// # }
/// ```
///
/// ## Streaming response
/// ```no_run
/// # use fm_bindings::LanguageModelSession;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let session = LanguageModelSession::new()?;
/// session.stream_response("What is Rust?", |chunk| {
///     print!("{}", chunk);
/// })?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct LanguageModelSession {
    _private: (),
}

impl LanguageModelSession {
    /// Creates a new language model session
    ///
    /// This checks that the Foundation Model is available on the system.
    ///
    /// # Errors
    ///
    /// Returns `Error::ModelNotAvailable` if Apple Intelligence is not enabled
    /// or the system model is unavailable.
    pub fn new() -> Result<Self> {
        // Check availability before creating the session (fail-fast)
        let is_available = unsafe { ffi::fm_check_availability() };

        if !is_available {
            return Err(Error::ModelNotAvailable);
        }

        Ok(Self { _private: () })
    }

    /// Generates a complete response to the given prompt
    ///
    /// This method blocks until the entire response is generated and returned as a String.
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

// C Callbacks for response()

extern "C" fn response_callback(
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
        let state = Box::from_raw(user_data as *mut Arc<(Mutex<ResponseState>, Condvar)>);
        let state_arc = (*state).clone();
        drop(state); // Drop the Box, but Arc is still alive

        let (mutex, cvar) = &*state_arc;
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
        let state = Box::from_raw(user_data as *mut Arc<(Mutex<ResponseState>, Condvar)>);
        let state_arc = (*state).clone();
        drop(state); // Drop the Box, but Arc is still alive

        let (mutex, cvar) = &*state_arc;
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

// C Callbacks for stream_response()

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
