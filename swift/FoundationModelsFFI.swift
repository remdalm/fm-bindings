// FoundationModelsFFI.swift
// C FFI wrapper around Apple's FoundationModels framework
// Requires: macOS 26+ or iOS 26+ with Apple Intelligence enabled

import Foundation
import FoundationModels

// MARK: - Availability

@_cdecl("fm_check_availability")
public func fm_check_availability() -> Bool {
    if #available(macOS 26.0, iOS 26.0, *) {
        return SystemLanguageModel.default.availability == .available
    }
    return false
}

// MARK: - Session Lifecycle

@_cdecl("fm_create_session")
public func fm_create_session(
    instructions: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, iOS 26.0, *) else {
        return nil
    }

    let session: LanguageModelSession

    if let instructions = instructions {
        let instructionsStr = String(cString: instructions)
        session = LanguageModelSession(instructions: instructionsStr)
    } else {
        session = LanguageModelSession()
    }

    return Unmanaged.passRetained(session).toOpaque()
}

@_cdecl("fm_create_session_from_transcript")
public func fm_create_session_from_transcript(
    transcriptJson: UnsafePointer<CChar>
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, iOS 26.0, *) else {
        return nil
    }

    let jsonStr = String(cString: transcriptJson)

    guard let jsonData = jsonStr.data(using: .utf8),
        let transcript = try? JSONDecoder().decode(Transcript.self, from: jsonData)
    else {
        return nil
    }

    let session = LanguageModelSession(transcript: transcript)
    return Unmanaged.passRetained(session).toOpaque()
}

@_cdecl("fm_destroy_session")
public func fm_destroy_session(sessionPtr: UnsafeMutableRawPointer) {
    // If we have a valid sessionPtr, fm_create_session already verified availability.
    guard #available(macOS 26.0, iOS 26.0, *) else {
        preconditionFailure("fm_destroy_session called on unsupported OS - this indicates a bug")
    }

    Unmanaged<LanguageModelSession>.fromOpaque(sessionPtr).release()
}

// MARK: - Transcript

@_cdecl("fm_get_transcript_json")
public func fm_get_transcript_json(
    sessionPtr: UnsafeMutableRawPointer
) -> UnsafeMutablePointer<CChar>? {
    // If we have a valid sessionPtr, fm_create_session already verified availability.
    guard #available(macOS 26.0, iOS 26.0, *) else {
        preconditionFailure(
            "fm_get_transcript_json called on unsupported OS - this indicates a bug")
    }

    let session = Unmanaged<LanguageModelSession>.fromOpaque(sessionPtr).takeUnretainedValue()

    guard let jsonData = try? JSONEncoder().encode(session.transcript),
        let jsonString = String(data: jsonData, encoding: .utf8)
    else {
        return nil
    }

    return strdup(jsonString)
}

@_cdecl("fm_free_string")
public func fm_free_string(ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

// MARK: - Task Management for Cancellation

/// Thread-safe storage for active streaming tasks
private final class TaskManager: @unchecked Sendable {
    static let shared = TaskManager()

    private var tasks: [UnsafeMutableRawPointer: Task<Void, Never>] = [:]
    private let lock = NSLock()

    private init() {}

    func store(_ task: Task<Void, Never>, for session: UnsafeMutableRawPointer) {
        lock.lock()
        defer { lock.unlock() }
        tasks[session] = task
    }

    func remove(for session: UnsafeMutableRawPointer) {
        lock.lock()
        defer { lock.unlock() }
        tasks.removeValue(forKey: session)
    }

    func cancel(for session: UnsafeMutableRawPointer) {
        lock.lock()
        let task = tasks[session]
        lock.unlock()

        task?.cancel()
    }
}

// MARK: - Response Generation

@_cdecl("fm_session_response")
public func fm_session_response(
    sessionPtr: UnsafeMutableRawPointer,
    prompt: UnsafePointer<CChar>,
    userData: UnsafeMutableRawPointer,
    onChunk: @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    onDone: @convention(c) (UnsafeMutableRawPointer?) -> Void,
    onError: @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    // #available is required by the compiler to access FoundationModels APIs.
    // If we have a valid sessionPtr, fm_create_session already verified availability,
    // so this branch should never execute in correct usage.
    guard #available(macOS 26.0, iOS 26.0, *) else {
        preconditionFailure("fm_session_response called on unsupported OS - this indicates a bug")
    }

    let session = Unmanaged<LanguageModelSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let promptStr = String(cString: prompt)

    let task = Task {
        do {
            let response = try await session.respond(to: promptStr)

            // Check cancellation before sending response
            if Task.isCancelled {
                onDone(userData)
                return
            }

            // Send the complete response as a single chunk
            response.content.withCString { onChunk($0, userData) }
            onDone(userData)
        } catch {
            if !Task.isCancelled {
                error.localizedDescription.withCString { onError($0, userData) }
            } else {
                onDone(userData)
            }
        }

        TaskManager.shared.remove(for: sessionPtr)
    }

    TaskManager.shared.store(task, for: sessionPtr)
}

@_cdecl("fm_session_stream")
public func fm_session_stream(
    sessionPtr: UnsafeMutableRawPointer,
    prompt: UnsafePointer<CChar>,
    userData: UnsafeMutableRawPointer,
    onChunk: @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void,
    onDone: @convention(c) (UnsafeMutableRawPointer?) -> Void,
    onError: @convention(c) (UnsafePointer<CChar>?, UnsafeMutableRawPointer?) -> Void
) {
    // #available is required by the compiler to access FoundationModels APIs.
    // If we have a valid sessionPtr, fm_create_session already verified availability,
    // so this branch should never execute in correct usage.
    guard #available(macOS 26.0, iOS 26.0, *) else {
        preconditionFailure("fm_session_stream called on unsupported OS - this indicates a bug")
    }

    let session = Unmanaged<LanguageModelSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let promptStr = String(cString: prompt)

    let task = Task {
        do {
            let stream = session.streamResponse(to: promptStr)

            var previousContent = ""
            for try await partialResponse in stream {
                // Check for cancellation at each iteration
                if Task.isCancelled {
                    break
                }

                // Calculate the delta (new content since last chunk)
                let currentContent = partialResponse.content
                if currentContent.count > previousContent.count {
                    let delta = String(currentContent.dropFirst(previousContent.count))
                    delta.withCString { onChunk($0, userData) }
                    previousContent = currentContent
                }
            }

            onDone(userData)
        } catch {
            // Don't report error if we were cancelled
            if !Task.isCancelled {
                error.localizedDescription.withCString { onError($0, userData) }
            } else {
                onDone(userData)
            }
        }

        TaskManager.shared.remove(for: sessionPtr)
    }

    TaskManager.shared.store(task, for: sessionPtr)
}

@_cdecl("fm_session_cancel_stream")
public func fm_session_cancel_stream(sessionPtr: UnsafeMutableRawPointer) {
    TaskManager.shared.cancel(for: sessionPtr)
}
