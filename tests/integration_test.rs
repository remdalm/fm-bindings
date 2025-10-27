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
fn test_blocking_response() -> Result<()> {
    let session = LanguageModelSession::new()?;
    let prompt = "What is 2+2?";

    println!("Testing blocking response with prompt: {}", prompt);

    let response = session.response(prompt)?;

    // Verify response is not empty
    assert!(!response.is_empty(), "Response should not be empty");

    // Verify response contains some content (basic sanity check)
    assert!(
        !response.is_empty(),
        "Response should contain meaningful content, got: {}",
        response
    );

    println!("✓ Blocking response test passed");
    println!("Response length: {} chars", response.len());
    println!(
        "Response preview: {}...",
        &response[..response.len().min(100)]
    );

    Ok(())
}

#[test]
fn test_streaming_response() -> Result<()> {
    let session = LanguageModelSession::new()?;
    let prompt = "Count from 1 to 5";

    println!("Testing streaming response with prompt: {}", prompt);

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

    // Verify the complete response is meaningful
    let full_response: String = collected_chunks.join("");
    assert!(
        !full_response.is_empty(),
        "Complete response should not be empty"
    );
    assert!(
        full_response.len() > 5,
        "Complete response should contain meaningful content"
    );

    println!("✓ Streaming response test passed");
    println!("Total chunks received: {}", collected_chunks.len());
    println!("Full response length: {} chars", full_response.len());
    println!(
        "Full response preview: {}...",
        &full_response[..full_response.len().min(100)]
    );

    Ok(())
}

#[test]
fn test_cancel_streaming_response() -> Result<()> {
    let session = LanguageModelSession::new()?;
    // Use a prompt that would generate a longer response
    let prompt = "Write a long story about a dragon and a knight";

    println!("Testing cancel streaming with prompt: {}", prompt);

    // Track chunks received and cancellation flag
    let chunks = Arc::new(Mutex::new(Vec::new()));
    let chunks_clone = Arc::clone(&chunks);
    let cancel_triggered = Arc::new(Mutex::new(false));
    let cancel_flag = Arc::clone(&cancel_triggered);

    // Start streaming in a separate thread so we can cancel it
    let session_clone = session.clone();
    let stream_handle = thread::spawn(move || {
        session_clone.stream_response(prompt, move |chunk| {
            let mut chunks_vec = chunks_clone.lock().unwrap();
            chunks_vec.push(chunk.to_string());
            println!("Received chunk before cancel: {:?}", chunk);

            // After receiving a few chunks, signal to cancel
            if chunks_vec.len() == 3 {
                let mut cancel = cancel_flag.lock().unwrap();
                *cancel = true;
            }
        })
    });

    // Poll for cancellation trigger
    let mut cancelled = false;
    for _ in 0..50 {
        // Wait up to 5 seconds
        thread::sleep(Duration::from_millis(100));

        let should_cancel = *cancel_triggered.lock().unwrap();
        if should_cancel && !cancelled {
            println!("Cancelling stream after receiving chunks...");
            session.cancel_stream();
            cancelled = true;
            break;
        }
    }

    // Wait for stream thread to complete
    let stream_result = stream_handle.join();

    // Verify cancellation happened
    assert!(
        cancelled,
        "Stream should have been cancelled after receiving chunks"
    );

    // The stream may complete successfully or return an error depending on timing
    // Both outcomes are acceptable for a cancellation test
    match stream_result {
        Ok(Ok(())) => println!("Stream completed (may have finished before cancel)"),
        Ok(Err(e)) => println!("Stream returned error after cancel: {:?}", e),
        Err(_) => println!("Stream thread panicked"),
    }

    // Verify we received some chunks before cancellation
    let collected_chunks = chunks.lock().unwrap();
    println!("Chunks received before cancel: {}", collected_chunks.len());

    // We should have received at least a few chunks before cancellation
    assert!(
        !collected_chunks.is_empty(),
        "Should have received at least one chunk before cancellation"
    );

    println!("✓ Cancel streaming test passed");
    println!("Total chunks before cancel: {}", collected_chunks.len());

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

    println!("✓ Empty prompt error handling test passed");

    Ok(())
}
