//! Reviewed literals, independent of the writer/planner, with shared publication templates.
use crate::freeze_transition::Tree;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fr2-canonical");
const CONTRACT: &str =
    "docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/input-contract.json";

/// Independent complete structural expectation for the contract's snapshot field. Enumerate
/// raw predecessor Git leaves and hash their bytes; never call snapshot or packet renderers.
pub(super) fn baseline(root: &Path, parent: &str) -> Value {
    let source = crate::runtime_advertising::source::Source::tip(root, parent).unwrap();
    let entries = super::git(root, &["ls-tree", "-r", "-z", parent]);
    let files = entries
        .split('\0')
        .filter(|row| !row.is_empty())
        .filter_map(|row| {
            let (meta, path) = row.split_once('\t').unwrap();
            if !meta.starts_with("100644 ") && !meta.starts_with("100755 ") {
                return None;
            }
            if Path::new(path).starts_with("target")
                || Path::new(path).starts_with("docs/agents/.uaa-temp/runtime-follow-on/runs")
            {
                return None;
            }
            let bytes = source.optional_bytes(path).unwrap().unwrap();
            Some(json!({"path":path,"sha256":hex::encode(Sha256::digest(bytes))}))
        })
        .collect::<Vec<_>>();
    json!({"files":files})
}

/// Only provenance fields vary with the real ON1 producer and Git commit. Resolve them
/// from predecessor bytes, then independently hash the full literal expected P and O parts.
/// No expected value is copied from the candidate record, contract, or planner.
pub(super) fn onboarding_bindings(root: &Path, parent: &str) -> BTreeMap<String, String> {
    let source = crate::runtime_advertising::source::Source::tip(root, parent).unwrap();
    let approval_path =
        "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
    let approval = source.read(approval_path).unwrap();
    let parsed: Value = toml_edit::de::from_str(&approval).unwrap();
    let registry: Value = toml_edit::de::from_str(
        &source
            .read(crate::agent_registry::REGISTRY_RELATIVE_PATH)
            .unwrap(),
    )
    .unwrap();
    let agent = registry["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["agent_id"] == "depth_fixture")
        .unwrap();
    let mut bindings = BTreeMap::from([
        (
            "{{APPROVAL_COMMIT}}".into(),
            parsed["approval_commit"].as_str().unwrap().into(),
        ),
        (
            "{{APPROVAL_SHA256}}".into(),
            hex::encode(Sha256::digest(approval.as_bytes())),
        ),
        (
            "{{CLASSIFICATION_BASIS}}".into(),
            agent["support_depth"]["enrollments"][0]["operations"][0]["classification_basis"]
                .as_str()
                .unwrap()
                .into(),
        ),
    ]);
    let literal = std::fs::read_to_string(
        Path::new(FIXTURES).join("on2/cli_manifests/depth_fixture/reports/0.2.0/depth-record.json"),
    )
    .unwrap();
    let expected: Value = serde_json::from_str(&literal).unwrap();
    for row in expected["obligations"]["acquired_inputs"]
        .as_array()
        .unwrap()
    {
        let path = row["path"].as_str().unwrap();
        bindings.insert(
            format!("{{{{INPUT_SHA256:{path}}}}}"),
            hex::encode(Sha256::digest(
                source.optional_bytes(path).unwrap().unwrap(),
            )),
        );
    }
    for (part, token) in [
        ("policy", "{{POLICY_IDENTITY}}"),
        ("obligations", "{{OBLIGATIONS_IDENTITY}}"),
    ] {
        let mut expanded = literal.clone();
        for (key, value) in &bindings {
            expanded = expanded.replace(key, value);
        }
        let mut expected: Value = serde_json::from_str(&expanded).unwrap();
        expected[part].as_object_mut().unwrap().remove("identity");
        bindings.insert(
            token.into(),
            hex::encode(Sha256::digest(sorted_json(&expected[part]))),
        );
    }
    bindings
}

fn sorted_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map.iter().collect();
            format!(
                "{{{}}}",
                sorted
                    .into_iter()
                    .map(|(k, v)| format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap(),
                        sorted_json(v)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(sorted_json).collect::<Vec<_>>().join(",")
        ),
        _ => serde_json::to_string(value).unwrap(),
    }
}

pub(super) fn check(
    lane: &str,
    tree: &Tree,
    baseline: Option<Value>,
    bindings: &BTreeMap<String, String>,
) {
    let inventory: BTreeMap<String, BTreeMap<String, String>> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/fr2-canonical/inventory.json"
    ))
    .unwrap();
    let expected = &inventory[lane];
    let selected = |path: &str| {
        path.starts_with("docs/agents/lifecycle/depth_fixture-maintenance/")
            || path.starts_with("docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/")
            || path == crate::support_matrix::JSON_OUTPUT_PATH
            || path == crate::support_matrix::MARKDOWN_OUTPUT_PATH
            || path == crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH
            || path == "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json"
            || path == "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json"
    };
    assert_eq!(
        tree.keys().filter(|p| selected(p)).collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "{lane} canonical inventory"
    );
    for (path, fixture) in expected {
        let mut literal = std::fs::read_to_string(Path::new(FIXTURES).join(fixture)).unwrap();
        if fixture == "common/support.json" || fixture == "common/support-matrix.md" {
            let name = fixture.strip_prefix("common/").unwrap();
            let rows = std::fs::read_to_string(
                Path::new(FIXTURES).join(lane).join(format!("{name}.rows")),
            )
            .unwrap();
            literal = literal.replace("{{DEPTH_FIXTURE_ROWS}}", &rows);
        }
        for (token, value) in bindings {
            literal = literal.replace(token, value);
        }
        let actual = &tree[path];
        assert_eq!((&*actual.mode, &*actual.kind), ("100644", "blob"), "{path}");
        if path == CONTRACT {
            let mut expected: Value = serde_json::from_str(&literal).unwrap();
            assert!(expected["baseline"].is_null());
            expected["baseline"] = baseline.clone().expect("ON2 predecessor baseline");
            assert_eq!(
                serde_json::from_slice::<Value>(&actual.bytes).unwrap(),
                expected,
                "{path}"
            );
        } else {
            assert_eq!(
                std::str::from_utf8(&actual.bytes).unwrap(),
                literal,
                "{lane}: {path}"
            );
        }
    }
}
