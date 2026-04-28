# Changelog

All notable changes to this project will be documented in this file.

## [0.1.5] - 2026-04-28

### Changed

- **macOS Swift bridge**: Use a static Swift bridge archive when the Rust deployment target is 26.0 or higher
- **Swift concurrency linking**: Link static macOS builds against a uniquely named Swift concurrency stub to avoid downstream `@rpath` requirements
- **Build fallback**: Keep macOS deployment targets below 26.0 on the dylib bridge and emit a build warning explaining how to enable the static bridge

## [0.1.4] - 2026-01-30

### Changed

- **Build checks**: Fail fast on unsupported targets and require macOS/iOS deployment targets >= 26.0
- **Swift compilation**: Resolve SDK paths via `xcrun`, pass explicit `-sdk`/`-target`, and archive iOS objects with `libtool`
- **Linking**: Link directives now come from `build.rs` instead of a static `#[link]` attribute

## [0.1.3] - 2025-12-03

### Added

- **Session persistence**: Save and restore sessions with `transcript_json()` and `from_transcript_json()`
- **Instructions support**: Create sessions with system prompts using `with_instructions()`
- **Thread safety**: `LanguageModelSession` now implements `Send + Sync`

### Changed

- **Session state**: Sessions now maintain conversation history and can be persisted
- **Response methods**: `response()` and `stream_response()` now update the session transcript
