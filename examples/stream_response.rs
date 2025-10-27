// examples/stream_response.rs
// Example: Streaming response generation
//
// This example demonstrates using the `stream_response()` method to get
// real-time incremental updates from the Foundation Model. This provides
// a much better user experience with immediate feedback.
//
// Usage: cargo run --example stream_response

use fm_bindings::LanguageModelSession;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Foundation Models - Streaming Response Example ===\n");

    // Create a new session
    println!("Creating session...");
    let session = LanguageModelSession::new()?;
    println!("Session created!\n");

    // Define the prompt
    let prompt = "Tell me a short story about a robot learning to paint.";
    println!("Prompt: \"{}\"\n", prompt);
    println!("Streaming response:\n");
    println!("---");

    // Stream the response chunk by chunk
    // The callback is called for each chunk as it's generated
    session.stream_response(prompt, |chunk| {
        // Print each chunk immediately
        print!("{}", chunk);

        // Flush stdout to ensure immediate display
        io::stdout().flush().unwrap();
    })?;

    // Print completion message
    println!("\n---");
    println!("\n=== Stream Complete ===");

    Ok(())
}
