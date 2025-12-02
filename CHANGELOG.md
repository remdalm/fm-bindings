# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - Unreleased

### Added

- **Session persistence**: Save and restore sessions with `transcript_json()` and `from_transcript_json()`
- **Instructions support**: Create sessions with system prompts using `with_instructions()`
- **Thread safety**: `LanguageModelSession` now implements `Send + Sync`

### Changed

- **Session state**: Sessions now maintain conversation history and can be persisted
- **Response methods**: `response()` and `stream_response()` now update the session transcript
