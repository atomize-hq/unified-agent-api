// This file is derived from committed repo truth.
// Validate it with `cargo test -p xtask --all-targets`.

#[cfg(feature = "codex")]
const CODEX_RUNTIME_SUPPORT: &[EmbeddedRuntimeSupportRecord] = &[
    EmbeddedRuntimeSupportRecord {
        target_triple: "aarch64-apple-darwin",
        latest_validated: None,
    },
    EmbeddedRuntimeSupportRecord {
        target_triple: "x86_64-pc-windows-msvc",
        latest_validated: None,
    },
    EmbeddedRuntimeSupportRecord {
        target_triple: "x86_64-unknown-linux-musl",
        latest_validated: Some("0.125.0"),
    },
];
