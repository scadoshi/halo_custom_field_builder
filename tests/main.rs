//! Runs the built program in a scratch directory and checks how it reports a failure.
#![expect(
    clippy::unwrap_used,
    reason = "a test: its helpers panic when setup fails"
)]

use std::path::PathBuf;
use std::process::{Command, Output};

/// An empty scratch directory unique to `name` and this process.
fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("halo_main_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs the program in `dir` with an empty environment. `BASE_URL` is plain
/// http, so the config refuses it even if a `.env` turns up, and nothing
/// reaches the network.
fn run_in(dir: &PathBuf) -> Output {
    Command::new(env!("CARGO_BIN_EXE_halo_custom_field_builder"))
        .current_dir(dir)
        .env_clear()
        .env("BASE_URL", "http://127.0.0.1:9")
        .output()
        .unwrap()
}

#[test]
fn a_failure_prints_the_whole_error_chain_and_exits_with_1() {
    let dir = scratch_dir("chain");
    let output = run_in(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    // No .env in the scratch directory: the context and its cause both show.
    assert!(
        stderr.contains("failed to load environment: "),
        "stderr: {stderr}"
    );
}

#[test]
fn a_failure_before_logging_starts_still_prints() {
    let dir = scratch_dir("no_logging");
    // A file where the logs directory goes stops logging from starting.
    std::fs::write(dir.join("logs"), "").unwrap();
    let output = run_in(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("failed to create logs directory"),
        "stderr: {stderr}"
    );
}
