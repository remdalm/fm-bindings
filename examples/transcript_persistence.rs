// Example: Persisting and restoring session transcripts
//
// This example demonstrates how to save a session's transcript to disk
// and restore it later to continue a conversation with full context.
//
// Usage: cargo run --example transcript_persistence

use fm_bindings::LanguageModelSession;
use std::fs;
use std::path::Path;

const TRANSCRIPT_FILE: &str = "session_transcript.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Foundation Models - Transcript Persistence Example ===\n");

    // Check if we have a saved session to restore
    if Path::new(TRANSCRIPT_FILE).exists() {
        println!("Found saved session, restoring...\n");
        restore_and_continue()?;
    } else {
        println!("No saved session found, starting fresh...\n");
        create_and_save()?;
    }

    Ok(())
}

fn create_and_save() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new session with instructions
    let session = LanguageModelSession::with_instructions(
        "You are a knowledgeable travel guide specializing in Japanese culture and tourism.",
    )?;
    println!("Created new session with travel guide persona.\n");

    // Have a conversation
    println!("User: Tell me about Tokyo.");
    let response1 = session.response("Tell me about Tokyo.")?;
    println!("Assistant: {}\n", response1);

    println!("User: What's the best time to visit?");
    let response2 = session.response("What's the best time to visit?")?;
    println!("Assistant: {}\n", response2);

    // Save the transcript
    let transcript_json = session.transcript_json()?;
    fs::write(TRANSCRIPT_FILE, &transcript_json)?;
    println!("--- Session saved to {} ---", TRANSCRIPT_FILE);
    println!("Run this example again to restore and continue the conversation.\n");

    Ok(())
}

fn restore_and_continue() -> Result<(), Box<dyn std::error::Error>> {
    // Load the saved transcript
    let transcript_json = fs::read_to_string(TRANSCRIPT_FILE)?;

    // Restore the session
    let session = LanguageModelSession::from_transcript_json(&transcript_json)?;
    println!("Session restored with previous conversation context.\n");

    // Continue the conversation - the model remembers the previous context
    println!("User: What about the food there?");
    let response = session.response("What about the food there?")?;
    println!("Assistant: {}\n", response);

    println!("User: Any restaurant recommendations?");
    let response2 = session.response("Any restaurant recommendations?")?;
    println!("Assistant: {}\n", response2);

    // Clean up the saved file
    fs::remove_file(TRANSCRIPT_FILE)?;
    println!("--- Cleaned up saved session ---\n");

    Ok(())
}
