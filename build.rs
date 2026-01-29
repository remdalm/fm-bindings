// build.rs
// Compiles Swift library and tells cargo how to link it
// Supports both macOS and iOS targets

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn parse_major_version(version: &str) -> Option<u32> {
    let major = version.trim().split('.').next()?;
    major.parse().ok()
}

fn main() {
    // Skip Swift compilation when building docs on docs.rs
    // docs.rs builds in a Linux container without Swift toolchain
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=Building on docs.rs - skipping Swift compilation");
        return;
    }

    // Rerun if any Swift source changes
    println!("cargo:rerun-if-changed=swift/");
    println!("cargo:rerun-if-env-changed=IPHONEOS_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    // Get target information
    let target = env::var("TARGET").expect("TARGET environment variable not set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set by cargo");

    // Detect platform
    let is_ios = target.contains("apple-ios");
    let is_ios_simulator = is_ios && target.contains("sim");
    let is_ios_device = is_ios && !is_ios_simulator;
    let is_macos = target.contains("apple-darwin");

    if !(is_ios_device || is_ios_simulator || is_macos) {
        panic!(
            "Unsupported target platform: {}. \
             This crate only supports iOS (device or simulator) and macOS targets with deployment target >= 26.0.",
            target
        );
    }

    let (deployment_var, deployment_target) = if is_macos {
        (
            "MACOSX_DEPLOYMENT_TARGET",
            env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "26.0".to_string()),
        )
    } else {
        (
            "IPHONEOS_DEPLOYMENT_TARGET",
            env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "26.0".to_string()),
        )
    };
    let deployment_major = parse_major_version(&deployment_target).unwrap_or(0);
    if deployment_major < 26 {
        panic!(
            "This crate requires {} >= 26.0 but {} is {}. \
Set {} to 26.0 or higher.",
            if is_macos { "macOS" } else { "iOS" },
            deployment_var,
            deployment_target,
            deployment_var
        );
    }

    println!("cargo:warning=Building for target: {}", target);

    // Configure based on platform
    let (lib_name, sdk_name, link_type) = if is_ios_device {
        ("libFoundationModelsFFI.a", Some("iphoneos"), "static")
    } else if is_ios_simulator {
        (
            "libFoundationModelsFFI.a",
            Some("iphonesimulator"),
            "static",
        )
    } else if is_macos {
        ("libFoundationModelsFFI.dylib", Some("macosx"), "dylib")
    } else {
        unreachable!("Unsupported target platform: {}", target);
    };

    let lib_path = PathBuf::from(&out_dir).join(lib_name);
    let obj_path = PathBuf::from(&out_dir).join("FoundationModelsFFI.o");

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
    let obj_path_str = if is_ios_device || is_ios_simulator {
        Some(
            obj_path
                .to_str()
                .expect("Object output path contains invalid UTF-8"),
        )
    } else {
        None
    };

    let mut cmd = Command::new("swiftc");
    if let Some(obj_path_str) = obj_path_str {
        cmd.args([
            "-emit-object",
            "-parse-as-library",
            "-o",
            obj_path_str,
            "-module-name",
            "FoundationModelsFFI",
            "swift/FoundationModelsFFI.swift",
        ]);
    } else {
        cmd.args([
            "-emit-library",
            "-o",
            lib_path_str,
            "-module-name",
            "FoundationModelsFFI",
            "swift/FoundationModelsFFI.swift",
        ]);
    }

    if env::var("PROFILE")
        .map(|profile| profile == "release")
        .unwrap_or(false)
    {
        cmd.arg("-O");
    }
    cmd.args(["-framework", "Foundation", "-framework", "FoundationModels"]);

    // Add SDK flag for Apple SDKs
    if let Some(sdk_name) = sdk_name {
        let sdk_output = Command::new("xcrun")
            .args(["--sdk", sdk_name, "--show-sdk-path"])
            .output()
            .expect("Failed to execute xcrun. Make sure Xcode command line tools are installed.");
        if !sdk_output.status.success() {
            panic!(
                "Failed to locate SDK {} via xcrun (status: {}).",
                sdk_name, sdk_output.status
            );
        }
        let sdk_path = String::from_utf8_lossy(&sdk_output.stdout);
        let sdk_path = sdk_path.trim();
        if sdk_path.is_empty() {
            panic!("xcrun returned an empty SDK path for {}.", sdk_name);
        }
        cmd.args(["-sdk", sdk_path]);
    }

    // Add target architecture for iOS/macOS
    if is_ios_device || is_ios_simulator {
        let swift_target = if target.starts_with("aarch64-") {
            if is_ios_simulator {
                format!("arm64-apple-ios{}-simulator", deployment_target)
            } else {
                format!("arm64-apple-ios{}", deployment_target)
            }
        } else if target.starts_with("x86_64-") {
            if is_ios_simulator {
                format!("x86_64-apple-ios{}-simulator", deployment_target)
            } else {
                format!("x86_64-apple-ios{}", deployment_target)
            }
        } else {
            target.clone()
        };
        cmd.args(["-target", &swift_target]);
    } else if is_macos {
        let swift_target = if target.starts_with("aarch64-") {
            format!("arm64-apple-macosx{}", deployment_target)
        } else if target.starts_with("x86_64-") {
            format!("x86_64-apple-macosx{}", deployment_target)
        } else {
            target.clone()
        };
        cmd.args(["-target", &swift_target]);
    }

    let status = cmd
        .status()
        .expect("Failed to execute swiftc. Make sure Swift is installed.");

    if !status.success() {
        panic!("Swift compilation failed for target: {}", target);
    }

    // For iOS builds, archive the object into a static library.
    if is_ios_device || is_ios_simulator {
        let obj_path_str = obj_path_str.expect("Object output path contains invalid UTF-8");

        let mut libtool = Command::new("xcrun");
        if let Some(sdk_name) = sdk_name {
            libtool.args(["--sdk", sdk_name]);
        }
        libtool.args(["libtool", "-static", "-o", lib_path_str, obj_path_str]);

        let status = libtool
            .status()
            .expect("Failed to execute libtool via xcrun.");
        if !status.success() {
            panic!(
                "libtool failed to create static library for target: {}",
                target
            );
        }
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
