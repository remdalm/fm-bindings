// build.rs
// Compiles Swift library and tells cargo how to link it
// Supports both macOS and iOS targets

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Rerun if the Swift source changes
    println!("cargo:rerun-if-changed=swift/FoundationModelsFFI.swift");

    // Get target information
    let target = env::var("TARGET").expect("TARGET environment variable not set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set by cargo");

    // Detect platform
    let is_ios_device = target.contains("aarch64-apple-ios") && !target.contains("sim");
    let is_ios_simulator = target.contains("apple-ios") && target.contains("sim");
    let is_macos = target.contains("apple-darwin");

    println!("cargo:warning=Building for target: {}", target);

    // Configure based on platform
    let (lib_name, _lib_extension, sdk_arg, link_type) = if is_ios_device {
        ("libFoundationModelsFFI.a", "a", "-sdk iphoneos", "static")
    } else if is_ios_simulator {
        (
            "libFoundationModelsFFI.a",
            "a",
            "-sdk iphonesimulator",
            "static",
        )
    } else if is_macos {
        ("libFoundationModelsFFI.dylib", "dylib", "", "dylib")
    } else {
        panic!(
            "Unsupported target platform: {}. \
             This crate only supports Apple platforms (macOS, iOS). \
             Build this on macOS or use cross-compilation with appropriate targets:\n\
             - aarch64-apple-ios (iOS device)\n\
             - aarch64-apple-ios-sim (iOS simulator)\n\
             - aarch64-apple-darwin (Apple Silicon macOS)\n\
             - x86_64-apple-darwin (Intel macOS)",
            target
        );
    };

    let lib_path = PathBuf::from(&out_dir).join(lib_name);

    // Step 1: Compile Swift library
    println!(
        "cargo:warning=Compiling Swift library for {}...",
        if is_ios_device {
            "iOS device"
        } else if is_ios_simulator {
            "iOS simulator"
        } else {
            "macOS"
        }
    );

    let lib_path_str = lib_path
        .to_str()
        .expect("Output path contains invalid UTF-8");

    let mut cmd = Command::new("swiftc");
    cmd.args([
        "-emit-library",
        "-o",
        lib_path_str,
        "-module-name",
        "FoundationModelsFFI",
        "swift/FoundationModelsFFI.swift",
        "-framework",
        "Foundation",
        "-framework",
        "FoundationModels",
    ]);

    // Add SDK flag for iOS builds
    if !sdk_arg.is_empty() {
        cmd.arg(sdk_arg);
    }

    // Add target architecture for iOS
    if is_ios_device || is_ios_simulator {
        cmd.arg("-target");
        cmd.arg(&target);
    }

    let status = cmd
        .status()
        .expect("Failed to execute swiftc. Make sure Swift is installed.");

    if !status.success() {
        panic!("Swift compilation failed for target: {}", target);
    }

    // Step 2: Configure linking
    // Tell cargo to link against our Swift library
    println!("cargo:rustc-link-lib={}=FoundationModelsFFI", link_type);

    // Tell cargo where to find the library (in OUT_DIR)
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Link system frameworks (available on both iOS and macOS)
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=FoundationModels");
}
