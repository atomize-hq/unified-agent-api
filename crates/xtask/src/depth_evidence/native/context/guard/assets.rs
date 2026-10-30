//! Only pinned, nonsecret bundled assets. No general home/session inventory.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
const OBSERVATION: &str = include_str!(
    "../../../../../../../cli_manifests/codex/reports/0.156.1/pilot-runtime-context.json"
);
const PROTOCOL: &str = include_str!(
    "../../../../../../../cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json"
);
pub(super) fn features(value: &serde_json::Value) -> bool {
    let Ok(report) = serde_json::from_str::<serde_json::Value>(PROTOCOL) else {
        return false;
    };
    let Some(allowed) = report["config_schema"]["properties"]["features"]["properties"].as_object()
    else {
        return false;
    };
    value.as_object().is_some_and(|o| {
        o.iter().all(|(k, v)| {
            allowed.contains_key(k)
                && if k == "tool_registry" {
                    v.as_object().is_some_and(|o| {
                        o.iter().all(|(k, v)| {
                            matches!(
                                k.as_str(),
                                "error_on_tool_collisions" | "turn_metadata_includes_tool_info"
                            ) && v == false
                        })
                    })
                } else {
                    v == false
                }
        })
    })
}
pub(super) fn bundled(home: &Path) -> Result<(), Refusal> {
    let root = home.join("skills");
    match std::fs::symlink_metadata(&root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(Refusal::ForeignLoader),
        Ok(m) if !m.is_dir() || m.is_symlink() => return Err(Refusal::ForeignLoader),
        _ => (),
    }
    let observation: serde_json::Value =
        serde_json::from_str(OBSERVATION).map_err(|_| Refusal::ForeignLoader)?;
    let rows = observation["after_config_reads"]
        .as_array()
        .ok_or(Refusal::ForeignLoader)?;
    let expected: BTreeMap<_, _> = rows
        .iter()
        .filter_map(|r| {
            Some((
                r["path"].as_str()?.strip_prefix("home/skills/")?.to_owned(),
                r["sha256"].as_str()?.to_owned(),
            ))
        })
        .collect();
    let mut actual = BTreeSet::new();
    let mut dirs = vec![root.clone()];
    let mut seen = 0usize;
    while let Some(dir) = dirs.pop() {
        for item in std::fs::read_dir(dir).map_err(|_| Refusal::ForeignLoader)? {
            seen += 1;
            if seen > 4096 {
                return Err(Refusal::ForeignLoader);
            }
            let path = item.map_err(|_| Refusal::ForeignLoader)?.path();
            let m = std::fs::symlink_metadata(&path).map_err(|_| Refusal::ForeignLoader)?;
            let rel = path
                .strip_prefix(&root)
                .map_err(|_| Refusal::ForeignLoader)?
                .to_str()
                .ok_or(Refusal::ForeignLoader)?
                .to_owned();
            if m.is_symlink() {
                return Err(Refusal::ForeignLoader);
            }
            if m.is_dir() {
                if !expected.keys().any(|p| p.starts_with(&format!("{rel}/"))) {
                    return Err(Refusal::ForeignLoader);
                }
                dirs.push(path);
            } else if m.is_file() {
                // Reject unknown names before reading. Authentication paths are never traversed.
                let hash = expected.get(&rel).ok_or(Refusal::ForeignLoader)?;
                if m.len() > 1_048_576 {
                    return Err(Refusal::ForeignLoader);
                }
                use std::io::Read;
                let mut bytes = Vec::new();
                let mut file = std::fs::File::open(&path).map_err(|_| Refusal::ForeignLoader)?;
                if stamp(&file.metadata().map_err(|_| Refusal::ForeignLoader)?) != stamp(&m) {
                    return Err(Refusal::InputDrift);
                }
                file.by_ref()
                    .take(1_048_577)
                    .read_to_end(&mut bytes)
                    .map_err(|_| Refusal::ForeignLoader)?;
                use sha2::Digest;
                if bytes.len() > 1_048_576 || hex::encode(sha2::Sha256::digest(bytes)) != *hash {
                    return Err(Refusal::ForeignLoader);
                }
                actual.insert(rel);
            } else {
                return Err(Refusal::ForeignLoader);
            }
        }
    }
    if actual != expected.keys().cloned().collect() {
        return Err(Refusal::ForeignLoader);
    }
    Ok(())
}

pub(super) fn binary_matches(digest: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(PROTOCOL)
        .is_ok_and(|r| r["binary_sha256"].as_str() == Some(digest))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_features_are_pinned_nonsecret_false_values_not_arbitrary_keys() {
        assert!(features(
            &serde_json::json!({"unified_exec":false,"tool_registry":{"error_on_tool_collisions":false}})
        ));
        assert!(!features(&serde_json::json!({"sk-unknown-feature":false})));
        assert!(!features(&serde_json::json!({"unified_exec":"sk-secret"})));
        assert!(!features(&serde_json::json!({"unified_exec":true})));
        assert!(!features(
            &serde_json::json!({"tool_registry":{"unknown":false}})
        ));
    }
    #[test]
    fn foreign_skill_is_refused_without_reading_its_contents() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("skills/foreign")).unwrap();
        std::fs::write(root.path().join("skills/foreign/SKILL.md"), "sk-never-read").unwrap();
        assert_eq!(bundled(root.path()), Err(Refusal::ForeignLoader));
    }
}
