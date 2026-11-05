//! Shared by the lifecycle writer tests: every writer other than the onboarding depth freeze
//! preserves the lifecycle state's `support_depth` object byte for byte (OF-P2).

use std::{fs, path::Path};

use xtask::agent_lifecycle::{LifecycleDepth, LifecycleState};

pub const DEPTH_OBJECT_KEY: &str = "support_depth";

/// Adds a `support_depth` object to the lifecycle state at `path` and returns the object's bytes.
pub fn inject_depth_object(path: &Path) -> String {
    // Through the typed state, so the bytes are the ones every lifecycle writer writes.
    let mut state: LifecycleState =
        serde_json::from_slice(&fs::read(path).expect("read lifecycle state")).expect("parse");
    state.support_depth = Some(LifecycleDepth {
        policy_identity: "a".repeat(64),
        obligations_identity: Some("b".repeat(64)),
    });
    let mut bytes = serde_json::to_vec_pretty(&state).expect("serialize");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write lifecycle state");
    depth_object_bytes(path)
}

/// The bytes of the `support_depth` object as the lifecycle state at `path` writes it.
pub fn depth_object_bytes(path: &Path) -> String {
    let text = fs::read_to_string(path).expect("read lifecycle state");
    let start = text
        .find(&format!("\"{DEPTH_OBJECT_KEY}\": {{"))
        .unwrap_or_else(|| panic!("no `{DEPTH_OBJECT_KEY}` object in {}", path.display()));
    let end = start + text[start..].find('}').expect("object end") + 1;
    text[start..end].to_string()
}

/// Writes the `support_depth` object in a form no lifecycle writer writes (compact, members
/// reordered) and returns the whole file's bytes.
pub fn inject_noncanonical_depth_object(path: &Path) -> Vec<u8> {
    inject_depth_object(path);
    let text = fs::read_to_string(path).expect("read lifecycle state");
    let compact = format!(
        "\"{DEPTH_OBJECT_KEY}\":{{\"obligations_identity\":\"{}\",\"policy_identity\":\"{}\"}}",
        "b".repeat(64),
        "a".repeat(64)
    );
    let text = text.replacen(&depth_object_bytes(path), &compact, 1);
    fs::write(path, &text).expect("write lifecycle state");
    text.into_bytes()
}

/// A writer refused a noncanonical depth object for that reason and left the state as it was.
pub fn assert_refused_unchanged(path: &Path, before: &[u8], exit_code: i32, stderr: &str) {
    assert_ne!(exit_code, 0, "a noncanonical depth object must be refused");
    assert!(stderr.contains(DEPTH_OBJECT_KEY), "stderr:\n{stderr}");
    assert_eq!(fs::read(path).expect("read lifecycle state"), before);
}
