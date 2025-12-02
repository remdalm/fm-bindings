// Example: Blocking response generation with instructions
//
// This example demonstrates using the `response()` method to get a complete
// response from the Foundation Model, with custom instructions.
//
// Usage: cargo run --example response

use fm_bindings::LanguageModelSession;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Foundation Models - Blocking Response Example ===\n");

    // Create a new session with instructions
    println!("Creating session with instructions...");
    let session = LanguageModelSession::with_instructions(
        "You are a helpful assistant. Provide concise, accurate answers.",
    )?;
    println!("Session created!\n");

    // Define the prompt
    let prompt = "What is Rust programming language? Please explain in 2-3 sentences.";
    println!("Prompt: \"{}\"\n", prompt);
    println!("Generating response...\n");

    // Get the complete response
    // This blocks until the entire response is generated
    let response = session.response(prompt)?;

    // Print the response
    println!("Response:\n{}\n", response);
    println!("=== Complete ===");

    Ok(())
}
