// FoundationModelsFFI.swift
// C FFI wrapper around Apple's FoundationModels framework
// Requires: macOS 26+ or iOS 26+ with Apple Intelligence enabled

import Foundation
import FoundationModels

// MARK: - Global State
// We keep task references so we can cancel if needed
// In production, you'd manage multiple concurrent sessions with IDs
private var responseTask: Task<Void, Never>?
private var streamTask: Task<Void, Never>?

// Locks to serialize FFI calls and prevent race conditions
// when multiple threads call the FFI functions concurrently
private let responseLock = NSLock()
private let streamLock = NSLock()

// MARK: - C Function Pointer Types
// These match the callback signatures Rust will pass to us
public typealias ChunkCallbackWithData = @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
public typealias DoneCallbackWithData = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias ErrorCallbackWithData = @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void

// MARK: - Availability Check
/// Checks if the Foundation Model is available on this system
///
/// - Returns: true if the model is available, false otherwise
///
/// This checks SystemLanguageModel.default.isAvailable which requires:
/// - macOS 26+ or iOS 26+ with Apple Intelligence enabled
@_cdecl("fm_check_availability")
public func fm_check_availability() -> Bool {
    let systemModel = SystemLanguageModel.default
    return systemModel.isAvailable
}

// MARK: - Blocking Response
/// Generates a complete response from the Foundation Model (blocking mode)
///
/// - Parameters:
///   - prompt: C string with the user's prompt
///   - userData: Opaque pointer passed to all callbacks
///   - onChunk: Called for each chunk of text generated
///   - onDone: Called when generation completes successfully
///   - onError: Called if an error occurs (passes error message)
///
/// See: https://developer.apple.com/documentation/FoundationModels/LanguageModelSession/respond(to:)
@_cdecl("fm_response")
public func fm_response(
    _ prompt: UnsafePointer<CChar>?,
    _ userData: UnsafeMutableRawPointer?,
    _ onChunk: ChunkCallbackWithData?,
    _ onDone: DoneCallbackWithData?,
    _ onError: ErrorCallbackWithData?
) {
    // 1. Acquire lock to serialize FFI calls
    responseLock.lock()
    defer { responseLock.unlock() }

    // 2. Convert C string to Swift String
    guard let promptCStr = prompt,
          let promptString = String(utf8String: promptCStr) else {
        "Invalid prompt".withCString { cString in
            onError?(cString, userData)
        }
        return
    }

    // 3. Cancel any existing response task
    responseTask?.cancel()

    // 4. Create semaphore to block until async work completes
    let semaphore = DispatchSemaphore(value: 0)

    // 5. Start new async task to handle response generation
    // Note: Availability is checked at session creation time in Rust
    responseTask = Task {
        defer { semaphore.signal() }

        do {
            // 6. Create a session
            let session = LanguageModelSession()

            // 7. Use streamResponse to collect tokens (more responsive than respond())
            // We still deliver all tokens but signal completion only at the end
            let stream = session.streamResponse(to: promptString)

            var lastText = ""

            // 8. Iterate through the stream
            for try await snapshot in stream {
                // Check if task was cancelled
                if Task.isCancelled { break }

                // Extract the string content from the snapshot
                let currentText = snapshot.content

                // Extract only the new chunks (delta) since last update
                let newContent = String(currentText.dropFirst(lastText.count))
                lastText = currentText

                // 9. Call the Rust callback with the new chunk
                if !newContent.isEmpty {
                    newContent.withCString { cString in
                        onChunk?(cString, userData)
                    }
                }
            }

            // 10. Generation completed successfully
            onDone?(userData)

        } catch {
            // 11. Handle any errors during generation
            let errorMsg = "Generation error: \(error.localizedDescription)"
            errorMsg.withCString { cString in
                onError?(cString, userData)
            }
        }
    }

    // 12. Block until the async work completes
    semaphore.wait()

    // Lock is automatically released via defer when function returns
}

// MARK: - Streaming Response
/// Starts streaming a response from the Foundation Model
///
/// - Parameters:
///   - prompt: C string with the user's prompt
///   - userData: Opaque pointer passed to all callbacks
///   - onChunk: Called for each chunk of text generated
///   - onDone: Called when streaming completes successfully
///   - onError: Called if an error occurs (passes error message)
///
/// See: https://developer.apple.com/documentation/FoundationModels/LanguageModelSession/streamResponse(to:)
@_cdecl("fm_start_stream")
public func fm_start_stream(
    _ prompt: UnsafePointer<CChar>?,
    _ userData: UnsafeMutableRawPointer?,
    _ onChunk: ChunkCallbackWithData?,
    _ onDone: DoneCallbackWithData?,
    _ onError: ErrorCallbackWithData?
) {
    // 1. Acquire lock to serialize FFI calls
    streamLock.lock()
    defer { streamLock.unlock() }

    // 2. Convert C string to Swift String
    guard let promptCStr = prompt,
          let promptString = String(utf8String: promptCStr) else {
        "Invalid prompt".withCString { cString in
            onError?(cString, userData)
        }
        return
    }

    // 3. Cancel any existing stream
    streamTask?.cancel()

    // 4. Create semaphore to block until async work completes
    let semaphore = DispatchSemaphore(value: 0)

    // 5. Start new async task to handle streaming
    // Note: Availability is checked at session creation time in Rust
    streamTask = Task {
        defer { semaphore.signal() }

        do {
            // 6. Create a session
            let session = LanguageModelSession()

            // 7. Start streaming
            let stream = session.streamResponse(to: promptString)

            var lastText = ""

            // 8. Iterate through the stream
            for try await snapshot in stream {
                // Check if task was cancelled
                if Task.isCancelled { break }

                // Extract the string content from the snapshot
                let currentText = snapshot.content

                // Extract only the new chunks (delta) since last update
                let newContent = String(currentText.dropFirst(lastText.count))
                lastText = currentText

                // 9. Call the Rust callback with the new chunk
                if !newContent.isEmpty {
                    newContent.withCString { cString in
                        onChunk?(cString, userData)
                    }
                }
            }

            // 10. Stream completed successfully
            onDone?(userData)

        } catch {
            // 11. Handle any errors during streaming
            let errorMsg = "Streaming error: \(error.localizedDescription)"
            errorMsg.withCString { cString in
                onError?(cString, userData)
            }
        }
    }

    // 12. Block until the async work completes
    semaphore.wait()

    // Lock is automatically released via defer when function returns
}

// MARK: - Stop Streaming
/// Cancels the current streaming task
@_cdecl("fm_stop_stream")
public func fm_stop_stream() {
    streamTask?.cancel()
    streamTask = nil
}
