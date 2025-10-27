# Rust bindings for Apple's Foundation Models framework

## Overview
**Goal:** Offer a safe Rust interface to Apple's on-device Foundation Models so Rust applications can request blocking or streaming language model responses without leaving the Rust ecosystem.

**Architecture:** A Swift bridge (`swift/FoundationModelsFFI.swift`) is compiled at build time via `build.rs`, and the Rust `LanguageModelSession` wraps its callbacks through a zero-copy FFI layer with typed errors for availability, validation, and generation failures.

## Platform Support

This crate supports:
- **macOS 26+** (Apple Silicon and Intel)
- **iOS 26+** (device and simulator)

Both platforms require Apple Intelligence to be enabled.

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

## Legal
This project is independent and not affiliated with, endorsed by, or sponsored by Apple Inc.
Apple, macOS, iOS, Apple Intelligence, and Apple silicon are trademarks of Apple Inc., registered in the U.S. and other countries and regions. Use of these marks here is for identification only.
