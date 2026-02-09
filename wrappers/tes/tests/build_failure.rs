use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Returns the path to the cargo-tes binary (debug build).
fn cargo_tes_bin() -> PathBuf {
    // Built by `cargo test -p cargo-tes` which compiles the binary
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_cargo-tes"));
    assert!(path.exists(), "cargo-tes binary not found at {:?}", path);
    path
}

/// Returns the path to the build-fail fixture crate.
fn build_fail_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // wrappers/
    path.pop(); // project root
    path.push("tools/build-fail");
    assert!(path.exists(), "build-fail fixture not found at {:?}", path);
    path
}

#[test]
fn build_rs_failure_does_not_hang() {
    let start = Instant::now();
    let timeout = Duration::from_secs(60);

    let output = Command::new(cargo_tes_bin())
        .arg("tes")
        .current_dir(build_fail_dir())
        .output()
        .expect("failed to run cargo-tes");

    let elapsed = start.elapsed();

    // Must finish well within the timeout — if it hung, we'd never get here,
    // but this assertion documents the expectation.
    assert!(
        elapsed < timeout,
        "cargo-tes took {:?}, expected it to finish within {:?}",
        elapsed,
        timeout
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Vec<String> =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON array");

    // Should report the build failure exit code
    assert!(
        parsed
            .iter()
            .any(|s| s.contains("Cargo test failed with exit code")),
        "expected exit code error in output, got: {:?}",
        parsed
    );

    // Should capture the build.rs panic message from stderr
    assert!(
        parsed
            .iter()
            .any(|s| s.contains("build.rs") || s.contains("intentional")),
        "expected build.rs error details in output, got: {:?}",
        parsed
    );
}
