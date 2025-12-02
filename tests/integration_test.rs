//! Integration tests for fm-bindings
//!
//! # Platform Requirements
//!
//! These tests require:
//! - macOS 26+ or iOS 26+
//! - Apple Intelligence enabled
//! - Must be run on an Apple platform (will not compile on Linux/Windows)
//!
//! # Running the tests
//!
//! On macOS:
//! ```sh
//! cargo test
//! ```
//!
//! On iOS (requires a device or simulator):
//! ```sh
//! cargo test --target aarch64-apple-ios-sim
//! ```

use fm_bindings::{LanguageModelSession, Result};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_session_without_instructions() -> Result<()> {
    let session = LanguageModelSession::new()?;
    let prompt = "What is 2+2?";

    println!("Testing session without instructions: {}", prompt);

    let response = session.response(prompt)?;

    println!("Response: {}", response);
    assert!(!response.is_empty(), "Response should not be empty");
    Ok(())
}

#[test]
fn test_session_with_instructions() -> Result<()> {
    let session = LanguageModelSession::with_instructions(
        "You are a helpful math tutor. Always explain your reasoning step by step.",
    )?;
    let prompt = "What is 15% of 80?";

    let response = session.response(prompt)?;
    assert!(!response.is_empty(), "Response should not be empty");

    Ok(())
}

#[test]
fn test_multi_turn_conversation() -> Result<()> {
    let session = LanguageModelSession::with_instructions(
        "You are a helpful assistant. Remember context from previous messages.",
    )?;

    // First turn
    let response1 = session.response("My name is Alice.")?;
    assert!(!response1.is_empty());

    // Second turn - should remember the name
    let response2 = session.response("What is my name?")?;
    assert!(!response2.is_empty());
    println!("Turn 2 response: {}", response2);

    Ok(())
}

#[test]
fn test_transcript_persistence() -> Result<()> {
    // Create first session and have a conversation
    let session1 = LanguageModelSession::with_instructions("You are a helpful assistant.")?;
    let _ = session1.response("Remember the number 42.")?;

    // Get transcript JSON
    let transcript_json = session1.transcript_json()?;
    assert!(
        !transcript_json.is_empty(),
        "Transcript should not be empty"
    );

    // Create second session from transcript
    let session2 = LanguageModelSession::from_transcript_json(&transcript_json)?;

    // The new session should have context from the previous conversation
    let response = session2.response("What number did I ask you to remember?")?;
    assert!(!response.is_empty());
    println!("Restored session response: {}", response);

    Ok(())
}

#[test]
fn test_streaming_response() -> Result<()> {
    let session = LanguageModelSession::new()?;
    let prompt = "Count from 1 to 5";

    // Track chunks received
    let chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = Arc::clone(&chunks);

    // Stream response and collect chunks
    session.stream_response(prompt, move |chunk| {
        let mut chunks_vec = chunks_clone.lock().unwrap();
        chunks_vec.push(chunk.to_string());
        println!("Received chunk: {:?}", chunk);
    })?;

    // Verify we received chunks
    let collected_chunks = chunks.lock().unwrap();
    assert!(
        !collected_chunks.is_empty(),
        "Should have received at least one chunk"
    );

    println!("Total chunks received: {}", collected_chunks.len());

    // Verify the complete response is meaningful
    let full_response: String = collected_chunks.join("");
    assert!(
        !full_response.is_empty(),
        "Complete response should not be empty"
    );

    println!("Full response: {}", full_response);

    Ok(())
}

#[test]
fn test_cancel_streaming_response() -> Result<()> {
    let session = Arc::new(LanguageModelSession::with_instructions(
        "Write very long, detailed responses.",
    )?);
    let prompt = "Write a very long essay about the history of computing.";

    let chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = Arc::clone(&chunks);
    let cancel_triggered = Arc::new(Mutex::new(false));
    let cancel_flag = Arc::clone(&cancel_triggered);

    // Clone session for the spawned thread
    let session_for_stream = Arc::clone(&session);

    let stream_handle = thread::spawn(move || {
        session_for_stream.stream_response(prompt, move |chunk| {
            let mut chunks_vec = chunks_clone.lock().unwrap();
            chunks_vec.push(chunk.to_string());
            println!("{}", chunk);

            // After receiving a few chunks, signal to cancel
            if chunks_vec.len() == 3 {
                let mut cancel = cancel_flag.lock().unwrap();
                *cancel = true;
            }
        })
    });

    // Poll for cancellation trigger
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(100));

        let should_cancel = *cancel_triggered.lock().unwrap();
        if should_cancel {
            println!("Cancelling stream after receiving chunks...");
            session.cancel_stream();
            break;
        }
    }

    // Wait for stream thread to complete
    let _ = stream_handle.join();

    let collected_chunks = chunks.lock().unwrap();
    println!("Chunks received before cancel: {}", collected_chunks.len());

    Ok(())
}

#[test]
fn test_empty_prompt_error() -> Result<()> {
    let session = LanguageModelSession::new()?;

    // Test blocking response with empty prompt
    let result = session.response("");
    assert!(
        result.is_err(),
        "Empty prompt should return an error for blocking response"
    );

    // Test streaming response with empty prompt
    let result = session.stream_response("", |_| {});
    assert!(
        result.is_err(),
        "Empty prompt should return an error for streaming response"
    );

    Ok(())
}

#[test]
fn test_transcript_json_format() -> Result<()> {
    let session = LanguageModelSession::with_instructions("Test instructions")?;
    let _ = session.response("Hello")?;

    let json = session.transcript_json()?;

    // Verify it's valid JSON
    assert!(
        json.starts_with('[') || json.starts_with('{'),
        "Transcript should be valid JSON"
    );

    assert!(json.contains("Hello"));

    println!("JSON: {}", json);

    Ok(())
}
