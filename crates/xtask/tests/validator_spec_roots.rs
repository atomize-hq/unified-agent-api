//! Every registry manifest root has a validator spec, and every spec states the same "Depth
//! records" section (Annex B of `docs/specs/support-depth-contract.md` and the checks complete
//! mediation rule 7 requires). A spec says its root adopts Annex B only where the root's
//! `RULES.json` names the record path; a root that does not adopt it holds no depth record.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use xtask::agent_registry::AgentRegistry;

const CANONICAL: &str = include_str!("../data/agent_registry.toml");
const DEPTH_SECTION: &str = "## Depth records\n";
const SYNTHETIC_SPEC: &str =
    "tests/fixtures/depth-synthetic/governing/manifest-root/VALIDATOR_SPEC.md";
const NOT_ADOPTED: &str = "root has no `RULES.json`, so today it does not adopt Annex B";

/// The violation codes the shared section states. Each is one the validator or the support
/// publication emits.
const DEPTH_CODES: [&str; 20] = [
    "DEPTH_ACCEPTANCE_ENTRIES_CHANGED",
    "DEPTH_ACCEPTANCE_ENTRY_MISSING",
    "DEPTH_ACCEPTANCE_INPUT_INVALID",
    "DEPTH_APPROVAL_UNREADABLE",
    "DEPTH_LIFECYCLE_DANGLING",
    "DEPTH_RECORD_INVALID",
    "DEPTH_RECORD_MISSING",
    "DEPTH_RECORD_NOT_THE_REQUESTS",
    "DEPTH_RECORD_PATH_INVALID",
    "DEPTH_RECORD_REMOVED",
    "DEPTH_RECORD_UNADOPTED",
    "DEPTH_RECORD_UNANCHORED",
    "DEPTH_RECORD_WITHOUT_FREEZE",
    "DEPTH_REQUEST_UNREADABLE",
    "DEPTH_ROUTE_REFUSED",
    "DEPTH_SELECTION_SHRUNK",
    "DEPTH_TIP_UNREADABLE",
    "SUPPORT_MATRIX_DEPTH_MISMATCH",
    "SUPPORT_MATRIX_ROOT_READ_ERROR",
    "SUPPORT_MATRIX_SCHEMA_INVALID",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn depth_section(spec: &str, origin: &str) -> String {
    let start = spec
        .find(DEPTH_SECTION)
        .unwrap_or_else(|| panic!("{origin} has no `## Depth records` section"));
    spec[start..].to_string()
}

/// Whether `RULES.json` names `<root>/reports/<version>/depth-record.json`, which is how a root
/// adopts Annex B. A root without `RULES.json` adopts nothing.
fn adopts_annex_b(root: &Path, manifest_root: &str) -> bool {
    let Ok(text) = fs::read_to_string(root.join("RULES.json")) else {
        return false;
    };
    let rules: Value = serde_json::from_str(&text).expect("RULES.json parses");
    rules["storage"]["committed"]["reports"]["depth_record"].as_str()
        == Some(format!("{manifest_root}/reports/<version>/depth-record.json").as_str())
}

fn xtask_source() -> String {
    let mut out = String::new();
    let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push_str(&read(&path));
            }
        }
    }
    out
}

#[test]
fn every_registry_root_states_the_shared_depth_records_section() {
    let registry = AgentRegistry::parse(CANONICAL).unwrap();
    let synthetic = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(SYNTHETIC_SPEC));
    let shared = depth_section(&synthetic, SYNTHETIC_SPEC);
    for code in DEPTH_CODES {
        assert!(
            shared.contains(&format!("`{code}`")),
            "shared section omits {code}"
        );
    }
    let source = xtask_source();
    for code in DEPTH_CODES {
        assert!(
            source.contains(&format!("\"{code}\"")),
            "no xtask source emits {code}"
        );
    }

    let mut checked = Vec::new();
    for entry in &registry.agents {
        let root = repo_root().join(&entry.manifest_root);
        let spec_path = root.join("VALIDATOR_SPEC.md");
        let spec = read(&spec_path);
        let origin = format!("{}/VALIDATOR_SPEC.md", entry.manifest_root);
        assert!(
            spec.contains(&format!("`{}", entry.manifest_root)),
            "{origin} does not name its root"
        );
        assert_eq!(
            depth_section(&spec, &origin),
            shared,
            "{origin} must state the shared Depth records section word for word"
        );
        checked.push(entry.agent_id.clone());
    }
    assert_eq!(checked.len(), registry.agents.len());
}

#[test]
fn a_spec_claims_adoption_only_where_rules_name_the_record_path() {
    let registry = AgentRegistry::parse(CANONICAL).unwrap();
    for entry in &registry.agents {
        let root = repo_root().join(&entry.manifest_root);
        let spec = read(&root.join("VALIDATOR_SPEC.md"));
        let origin = format!("{}/VALIDATOR_SPEC.md", entry.manifest_root);
        if adopts_annex_b(&root, &entry.manifest_root) {
            assert!(!spec.contains(NOT_ADOPTED), "{origin} denies an adoption");
            continue;
        }
        // Record invariant 1: a root that does not adopt Annex B holds no depth record.
        assert!(
            xtask::depth_record::record_files(&root).is_empty(),
            "{} holds a depth record without adopting Annex B",
            entry.manifest_root
        );
        assert!(
            spec.contains("## Annex B adoption status\n") && spec.contains(NOT_ADOPTED),
            "{origin} must state that its root does not adopt Annex B"
        );
        assert!(
            spec.contains("## Open items\n"),
            "{origin} must list the truth decisions its root still needs"
        );
    }
}

/// A non-adopting root's spec restates its committed version metadata and pointers, so drift
/// between the spec and the files fails here rather than going unnoticed.
#[test]
fn a_non_adopting_spec_matches_its_committed_versions_and_pointers() {
    let registry = AgentRegistry::parse(CANONICAL).unwrap();
    for entry in &registry.agents {
        let root = repo_root().join(&entry.manifest_root);
        if adopts_annex_b(&root, &entry.manifest_root) {
            continue;
        }
        let spec = read(&root.join("VALIDATOR_SPEC.md"));
        let origin = format!("{}/VALIDATOR_SPEC.md", entry.manifest_root);
        for file in fs::read_dir(root.join("versions")).unwrap().flatten() {
            let name = file.file_name().to_string_lossy().into_owned();
            if !name.ends_with(".json") {
                continue;
            }
            let metadata: Value = serde_json::from_str(&read(&file.path())).unwrap();
            let status = metadata["status"].as_str().unwrap();
            assert!(
                spec.contains(&format!("`versions/{name}` exists with `status: {status}`")),
                "{origin} does not state `versions/{name}` with status `{status}`"
            );
        }
        for pointer in ["min_supported.txt", "latest_validated.txt"] {
            let value = read(&root.join(pointer)).trim().to_string();
            assert!(
                spec.contains(&format!("`{pointer}`")),
                "{origin}: {pointer}"
            );
            assert!(
                spec.contains(&format!("record `{value}`")),
                "{origin}: {value}"
            );
        }
        for target in &entry.canonical_targets {
            for kind in ["latest_supported", "latest_validated"] {
                let relative = format!("pointers/{kind}/{target}.txt");
                assert!(root.join(&relative).is_file(), "{relative} is missing");
                assert!(
                    spec.contains(&format!("`{relative}`")),
                    "{origin}: {relative}"
                );
            }
        }
    }
}
