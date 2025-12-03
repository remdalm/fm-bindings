# Rust bindings for Apple's Foundation Models framework

> **⚠️ Beta Warning**: This crate is currently in beta. Future updates may contain breaking changes to the API.

## Overview
**Goal:** Offer a safe Rust interface to Apple's on-device Foundation Models so Rust applications can request blocking or streaming language model responses without leaving the Rust ecosystem.

**Architecture:** A Swift bridge (`swift/FoundationModelsFFI.swift`) is compiled at build time via `build.rs`, and the Rust `LanguageModelSession` wraps its callbacks through a zero-copy FFI layer with typed errors for availability, validation, and generation failures.

## Platform Support

This crate supports:
- **macOS 26+** (Apple Silicon and Intel)
- **iOS 26+** (device and simulator)

Both platforms require Apple Intelligence to be enabled.

## Quick Start

Here's a simple example to get started:

```rust
use fm_bindings::LanguageModelSession;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a session
    let session = LanguageModelSession::new()?;

    // Get a response
    let response = session.response("What is Rust?")?;
    println!("{}", response);

    Ok(())
}
```

For more examples, see the [examples](./examples) directory.

## Features

- **Blocking responses** - Get complete responses with `response()`
- **Streaming responses** - Real-time token streaming with `stream_response()`
- **Stream cancellation** - Cancel ongoing streams with `cancel_stream()`
- **System instructions** - Configure model behavior with custom prompts
- **Session persistence** - Save and restore conversation context

## Usage Examples

### Basic Response
```rust
let session = LanguageModelSession::new()?;
let response = session.response("Explain Rust ownership")?;
println!("{}", response);
```

### With Instructions
```rust
let session = LanguageModelSession::with_instructions(
    "You are a helpful coding assistant."
)?;
let response = session.response("How do I handle errors in Rust?")?;
```

### Streaming Response
```rust
use std::io::{self, Write};

let session = LanguageModelSession::new()?;
session.stream_response("Tell me a story", |chunk| {
    print!("{}", chunk);
    io::stdout().flush().unwrap();
})?;
```

### Session Persistence
```rust
// Save session
let transcript = session.transcript_json()?;
std::fs::write("session.json", &transcript)?;

// Restore later
let saved = std::fs::read_to_string("session.json")?;
let session = LanguageModelSession::from_transcript_json(&saved)?;
```

For complete examples, see the [examples](./examples) directory:
- [`response.rs`](./examples/response.rs) - Blocking response generation
- [`stream_response.rs`](./examples/stream_response.rs) - Streaming responses
- [`transcript_persistence.rs`](./examples/transcript_persistence.rs) - Save/restore sessions

## Error Handling

The crate uses a custom `Error` type with these variants:

```rust
use fm_bindings::{LanguageModelSession, Error};

match session.response("Hello") {
    Ok(response) => println!("{}", response),
    Err(Error::ModelNotAvailable) => {
        eprintln!("Apple Intelligence is not enabled. Please enable it in System Settings.");
    }
    Err(Error::InvalidInput(msg)) => {
        eprintln!("Invalid input: {}", msg);
    }
    Err(Error::GenerationError(msg)) => {
        eprintln!("Generation failed: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Building

### macOS
On macOS, simply run:
```bash
cargo build
```

### iOS
To build for iOS, you need to specify the target:

```bash
# iOS device (ARM64)
cargo build --target aarch64-apple-ios

# iOS simulator (ARM64, for Apple Silicon Macs)
cargo build --target aarch64-apple-ios-sim

# iOS simulator (x86_64, for Intel Macs)
cargo build --target x86_64-apple-ios
```

### Cross-compilation Notes
- This crate **must** be built on macOS as it requires the Swift compiler and Apple SDKs
- The build script automatically detects the target platform and configures the appropriate SDK and library type
- macOS builds use dynamic libraries (`.dylib`)
- iOS builds use static libraries (`.a`)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Legal
This project is independent and not affiliated with, endorsed by, or sponsored by Apple Inc.
Apple, macOS, iOS, Apple Intelligence, and Apple silicon are trademarks of Apple Inc., registered in the U.S. and other countries and regions. Use of these marks here is for identification only.
