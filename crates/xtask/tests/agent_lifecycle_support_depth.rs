//! The lifecycle state's depth fields (OF-P2 and OF-P9 check (i) of
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`): typed, refusing unknown
//! keys, and preserved by the lifecycle writer.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};
use xtask::agent_lifecycle::{write_lifecycle_state, LifecycleDepth, LifecycleState};

const SHA: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const OTHER: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const STATE: &str = "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn committed_states() -> Vec<PathBuf> {
    let root = repo_root().join("docs/agents/lifecycle");
    let mut states: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap()
        .flatten()
        .map(|pack| pack.path().join("governance/lifecycle-state.json"))
        .filter(|path| path.is_file())
        .collect();
    states.sort();
    states
}

fn sample() -> Value {
    serde_json::from_slice(&fs::read(repo_root().join(STATE)).unwrap()).unwrap()
}

fn state(value: &Value) -> Result<LifecycleState, String> {
    parse(&value.to_string())
}

fn parse(text: &str) -> Result<LifecycleState, String> {
    serde_json::from_str(text).map_err(|err| err.to_string())
}

#[test]
fn every_committed_lifecycle_state_parses_and_is_written_back_unchanged() {
    let states = committed_states();
    assert!(states.len() >= 5, "found {states:?}");
    for path in states {
        let bytes = fs::read(&path).unwrap();
        let parsed: LifecycleState = serde_json::from_slice(&bytes)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        parsed
            .validate()
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert_eq!(parsed.support_depth, None, "{}", path.display());
        assert_eq!(parsed.written_paths_sha256, None, "{}", path.display());
        let mut written = serde_json::to_vec_pretty(&parsed).unwrap();
        written.push(b'\n');
        assert_eq!(
            String::from_utf8(written).unwrap(),
            String::from_utf8(bytes).unwrap(),
            "{} is not written back byte for byte",
            path.display()
        );
    }
}

#[test]
fn unknown_keys_are_refused_at_every_level() {
    let mut top = sample();
    top["support_depht"] = json!({ "policy_identity": SHA });
    assert!(state(&top).unwrap_err().contains("unknown field"));

    let mut summary = sample();
    summary["implementation_summary"]["extra"] = json!(true);
    assert!(state(&summary).unwrap_err().contains("unknown field"));

    let mut deferred = sample();
    if let Some(list) = deferred["implementation_summary"]["deferred_surfaces"].as_array_mut() {
        list.push(json!({ "surface": "add_dirs", "reason": "r", "extra": 1 }));
        assert!(state(&deferred).unwrap_err().contains("unknown field"));
    }

    let mut object = sample();
    object["support_depth"] = json!({ "policy_identity": SHA, "version": "1.2.3" });
    assert!(state(&object).unwrap_err().contains("unknown field"));
}

#[test]
fn the_depth_object_and_written_paths_digest_are_typed_and_checked() {
    let mut value = sample();
    value["support_depth"] = json!({ "policy_identity": SHA });
    let parsed = state(&value).unwrap();
    parsed.validate().expect("P frozen, O not yet");
    assert_eq!(
        parsed.support_depth.as_deref(),
        Some(&LifecycleDepth {
            policy_identity: SHA.to_string(),
            obligations_identity: None
        })
    );
    // Written as read: the object, and no `null` for the O not yet frozen.
    assert_eq!(serde_json::to_value(&parsed).unwrap(), value);

    value["support_depth"]["obligations_identity"] = json!("not a digest");
    let err = state(&value).unwrap().validate().unwrap_err().to_string();
    assert!(err.contains("support_depth.obligations_identity"), "{err}");

    // The digest of OF-P9 check (i) is the depth-enrolled write run's, recorded at success.
    let mut digest = sample();
    digest["written_paths_sha256"] = json!(SHA);
    let err = state(&digest).unwrap().validate().unwrap_err().to_string();
    assert!(err.contains("depth-enrolled"), "{err}");
    digest["support_depth"] = json!({ "policy_identity": SHA, "obligations_identity": OTHER });
    state(&digest)
        .unwrap()
        .validate()
        .expect("closed baseline with digest");
    digest["written_paths_sha256"] = json!("ABC");
    assert!(state(&digest).unwrap().validate().is_err());

    // Before the write run passes, there is no digest to record.
    let enrolled = ["registry_entry", "docs_pack", "manifest_root_skeleton"];
    let mut early = sample();
    early["lifecycle_stage"] = json!("enrolled");
    early["support_tier"] = json!("bootstrap");
    early["required_evidence"] = json!(enrolled);
    early["satisfied_evidence"] = json!(enrolled);
    for field in [
        "implementation_summary",
        "publication_packet_path",
        "publication_packet_sha256",
        "closeout_baseline_path",
    ] {
        early[field] = Value::Null;
    }
    early
        .as_object_mut()
        .unwrap()
        .retain(|key, _| !key.starts_with("maintenance_readiness"));
    early["support_depth"] = json!({ "policy_identity": SHA });
    state(&early)
        .unwrap()
        .validate()
        .expect("enrolled with P frozen");
    early["written_paths_sha256"] = json!(SHA);
    let err = state(&early).unwrap().validate().unwrap_err().to_string();
    assert!(err.contains("written_paths_sha256"), "{err}");
}

fn write_state(root: &Path, value: &Value) -> Result<(), String> {
    write_lifecycle_state(root, STATE, &state(value)?).map_err(|err| err.to_string())
}

fn seed(root: &Path, value: &Value) {
    let path = root.join(STATE);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    // Through the typed state, as a lifecycle writer writes it.
    let mut bytes = serde_json::to_vec_pretty(&state(value).unwrap()).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

#[test]
fn the_lifecycle_writer_preserves_the_depth_fields_and_refuses_to_change_them() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let mut depth = sample();
    depth["support_depth"] = json!({ "policy_identity": SHA, "obligations_identity": OTHER });
    depth["written_paths_sha256"] = json!(SHA);
    seed(root, &depth);

    // A writer that changes other fields and keeps both depth fields writes.
    let mut next = depth.clone();
    next["last_transition_by"] = json!("xtask some-writer --write");
    write_state(root, &next).expect("preserving write");
    let written: Value = serde_json::from_slice(&fs::read(root.join(STATE)).unwrap()).unwrap();
    assert_eq!(written["support_depth"], depth["support_depth"]);
    assert_eq!(
        written["written_paths_sha256"],
        depth["written_paths_sha256"]
    );

    // Removing, replacing or adding either field is refused, and the file is left as it was.
    let before = fs::read(root.join(STATE)).unwrap();
    let mut removed = next.clone();
    removed.as_object_mut().unwrap().remove("support_depth");
    removed
        .as_object_mut()
        .unwrap()
        .remove("written_paths_sha256");
    let mut replaced = next.clone();
    replaced["support_depth"]["policy_identity"] = json!(OTHER);
    let mut digest = next.clone();
    digest["written_paths_sha256"] = json!(OTHER);
    for refused in [removed, replaced, digest] {
        let err = write_state(root, &refused).unwrap_err();
        assert!(err.contains("preserved byte for byte"), "{err}");
        assert_eq!(fs::read(root.join(STATE)).unwrap(), before);
    }

    let plain = tempfile::tempdir().unwrap();
    seed(plain.path(), &sample());
    let err = write_state(plain.path(), &depth).unwrap_err();
    assert!(err.contains("preserved byte for byte"), "{err}");
    // No file yet: a new state may not carry a depth field either.
    let empty = tempfile::tempdir().unwrap();
    assert!(write_state(empty.path(), &depth).is_err());
    write_state(empty.path(), &sample()).expect("a state with no depth field");
}

/// A depth state whose object is written as `object` and whose digest as `digest`, both raw JSON.
fn depth_text(object: &str, digest: &str) -> String {
    let mut value = sample();
    value["support_depth"] = json!("OBJECT");
    value["written_paths_sha256"] = json!("DIGEST");
    format!("{}\n", serde_json::to_string_pretty(&value).unwrap())
        .replace("\"OBJECT\"", object)
        .replace("\"DIGEST\"", digest)
}

/// The depth fields in forms other than the one a lifecycle writer emits for a new value.
fn forms() -> Vec<(String, String)> {
    let object = |p: &str, o: &str| {
        format!("{{\"policy_identity\": \"{p}\", \"obligations_identity\": \"{o}\"}}")
    };
    vec![
        (object(SHA, OTHER), format!("\"{SHA}\"")),
        (
            format!("{{\"policy_identity\":\"{SHA}\",\"obligations_identity\":\"{OTHER}\"}}"),
            format!("\"{SHA}\""),
        ),
        (
            format!("{{\n\"obligations_identity\" : \"{OTHER}\",\n\"policy_identity\" : \"{SHA}\"\n}}"),
            format!("\"{SHA}\""),
        ),
        (
            format!("{{\"policy\\u005fidentity\": \"\\u0030{}\", \"obligations_identity\": \"{OTHER}\"}}", &SHA[1..]),
            format!("\"\\u0030{}\"", &SHA[1..]),
        ),
    ]
}

#[test]
fn the_depth_fields_are_read_by_a_json_parser_in_any_form() {
    for (object, digest) in forms() {
        let text = depth_text(&object, &digest);
        let parsed = parse(&text).unwrap_or_else(|err| panic!("{err}:\n{text}"));
        parsed.validate().expect("valid");
        assert_eq!(
            parsed.support_depth.as_deref().unwrap().policy_identity,
            SHA
        );
        assert_eq!(
            parsed
                .support_depth
                .as_deref()
                .unwrap()
                .obligations_identity
                .as_deref(),
            Some(OTHER)
        );
        assert_eq!(
            parsed.written_paths_sha256.as_deref().map(String::as_str),
            Some(SHA)
        );
        // An escaped member name is the same member.
        let escaped = text.replacen("\"support_depth\"", "\"support\\u005fdepth\"", 1);
        assert_eq!(
            parse(&escaped).unwrap().support_depth.as_deref(),
            parsed.support_depth.as_deref()
        );
    }
    // A repeated member is refused, at the root and in the object.
    let (object, digest) = forms().remove(0);
    let text = depth_text(&object, &digest);
    let repeated = [
        text.replacen(
            "\"support_depth\":",
            &format!("\"support_depth\": {object},\n  \"support\\u005fdepth\":"),
            1,
        ),
        text.replacen(
            "\"written_paths_sha256\":",
            &format!("\"written_paths_sha256\": {digest},\n  \"written_paths_sha256\":"),
            1,
        ),
        text.replacen(
            "{\"policy_identity\"",
            &format!("{{\"policy_identity\": \"{OTHER}\", \"policy_identity\""),
            1,
        ),
    ];
    for text in repeated {
        assert!(
            parse(&text).unwrap_err().contains("duplicate field"),
            "{text}"
        );
    }
}

#[test]
fn every_writer_writes_the_depth_fields_back_in_the_bytes_they_were_read_in() {
    for (object, digest) in forms() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join(STATE);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = depth_text(&object, &digest);
        fs::write(&path, &text).unwrap();
        let mut next = parse(&text).unwrap();
        next.last_transition_by = "xtask some-writer --write".to_string();
        write_lifecycle_state(fixture.path(), STATE, &next).expect("preserving write");
        let written = fs::read_to_string(&path).unwrap();
        assert!(
            written.contains(&format!("\"support_depth\": {object}")),
            "{written}"
        );
        assert!(
            written.contains(&format!("\"written_paths_sha256\": {digest}")),
            "{written}"
        );
        assert!(written.contains("xtask some-writer --write"));
    }
}

#[test]
fn the_lifecycle_writer_refuses_to_remove_escaped_depth_members() {
    let (object, digest) = forms().remove(0);
    let plain = parse(&depth_text(&object, &digest)).unwrap();
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join(STATE);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    for (escape, field) in [
        ("\"support\\u005fdepth\"", "support_depth"),
        ("\"written_paths\\u005fsha256\"", "written_paths_sha256"),
    ] {
        let text = depth_text(&object, &digest).replacen(&format!("\"{field}\""), escape, 1);
        fs::write(&path, &text).unwrap();
        let mut removed = plain.clone();
        match field {
            "support_depth" => {
                removed.support_depth = None;
                removed.written_paths_sha256 = None;
            }
            _ => removed.written_paths_sha256 = None,
        }
        let err = write_lifecycle_state(fixture.path(), STATE, &removed)
            .unwrap_err()
            .to_string();
        assert!(err.contains("preserved byte for byte"), "{field}: {err}");
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
    }
    // A destination whose depth members cannot be read is not overwritten.
    let text = depth_text(&object, &digest);
    let repeated = text.replacen(
        "\"support_depth\":",
        &format!("\"support_depth\": {object},\n  \"support_depth\":"),
        1,
    );
    fs::write(&path, &repeated).unwrap();
    let err = write_lifecycle_state(fixture.path(), STATE, &plain)
        .unwrap_err()
        .to_string();
    assert!(err.contains("cannot be read"), "{err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), repeated);
}
