//! Shared by the lifecycle writer tests: every writer other than the onboarding depth freeze
//! preserves the lifecycle state's `support_depth` object byte for byte (OF-P2).
#![allow(dead_code)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use xtask::agent_lifecycle::{LifecycleDepth, LifecycleState, Preserved};

pub const DEPTH_OBJECT_KEY: &str = "support_depth";

/// Adds a `support_depth` object, as a lifecycle writer writes a new one, to the lifecycle state at
/// `path` and returns the object's bytes.
pub fn inject_depth_object(path: &Path) -> String {
    let mut state: LifecycleState =
        serde_json::from_slice(&fs::read(path).expect("read lifecycle state")).expect("parse");
    state.support_depth = Some(
        Preserved::new(LifecycleDepth {
            policy_identity: "a".repeat(64),
            obligations_identity: Some("b".repeat(64)),
        })
        .expect("depth object"),
    );
    let mut bytes = serde_json::to_vec_pretty(&state).expect("serialize");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write lifecycle state");
    depth_object_bytes(path)
}

/// The bytes of the `support_depth` member as the lifecycle state at `path` holds it.
pub fn depth_object_bytes(path: &Path) -> String {
    let text = fs::read_to_string(path).expect("read lifecycle state");
    let start = text
        .find(&format!("\"{DEPTH_OBJECT_KEY}\": {{"))
        .unwrap_or_else(|| panic!("no `{DEPTH_OBJECT_KEY}` object in {}", path.display()));
    let end = start + text[start..].find('}').expect("object end") + 1;
    text[start..end].to_string()
}

/// Adds a `support_depth` object in a form no writer emits for a new one (compact, members
/// reordered, an escaped member name and character) and returns the object's exact bytes.
pub fn inject_noncanonical_depth_object(path: &Path) -> String {
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("read lifecycle state")).expect("parse");
    state[DEPTH_OBJECT_KEY] = serde_json::json!("OBJECT");
    let object = format!(
        "{{\"obligations_identity\":\"{}\",\"policy\\u005fidentity\" : \"\\u0061{}\"}}",
        "b".repeat(64),
        "a".repeat(63)
    );
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&state).expect("serialize")
    )
    .replacen("\"OBJECT\"", &object, 1);
    fs::write(path, &text).expect("write lifecycle state");
    let parsed: LifecycleState = serde_json::from_str(&text).expect("a valid lifecycle state");
    assert_eq!(
        parsed.support_depth.expect("object").policy_identity,
        "a".repeat(64)
    );
    object
}

/// The lifecycle state at `path` still holds the `support_depth` object in exactly `object`.
pub fn assert_depth_preserved(path: &Path, object: &str) {
    let text = fs::read_to_string(path).expect("read lifecycle state");
    assert!(
        text.contains(&format!("\"{DEPTH_OBJECT_KEY}\": {object}")),
        "the depth object is not preserved byte for byte:\n{text}"
    );
}

/// Every file under `root`, by path, for checking that a refused command wrote nothing.
pub fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.insert(path.clone(), fs::read(&path).unwrap_or_default());
            }
        }
    }
    files
}
