//! Shared helpers for integration tests that run Cargo-built fake binaries.

use std::path::{Path, PathBuf};

/// Cargo profile directory (`target/<profile>`) holding the uplifted workspace binaries,
/// resolved from the running test executable. Handles both the legacy layout
/// (`<profile>/deps/<test>`) and the build-dir layout Cargo uses from Rust 1.100
/// (`<profile>/build/<package>/<hash>/out/<test>`).
pub(crate) fn cargo_profile_dir() -> PathBuf {
    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    current_exe
        .ancestors()
        .skip(1)
        .find_map(|dir| match dir.file_name()?.to_str()? {
            "deps" | "build" => dir.parent().map(Path::to_path_buf),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("test binary {current_exe:?} is not under a Cargo deps/ or build/ directory")
        })
}
