//! The generator's rows for a root that holds a depth record (child module of
//! `support_matrix_depth.rs`): the rows the record implies, the facts it decides, and the records
//! publication refuses.

use std::{fs, path::Path};

use xtask::depth_record::file_sha256;
use xtask::support_matrix::derive_rows_for_test_roots;

use super::*;

const ROOT: &str = "cli_manifests/aider";

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// An aider root that expects linux only and has metadata and a clean coverage report for
/// `1.2.3` there. Its `RULES.json` adopts Annex B, and it holds `record` with every evidence file
/// bound at the content identity the workspace holds.
fn workspace(record: Value, accept: bool) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join(ROOT);
    let current = json!({ "expected_targets": [LINUX],
                          "inputs": [{ "target_triple": LINUX,
                                       "binary": { "semantic_version": "1.2.3" } }] });
    write(&root.join("current.json"), &current.to_string());
    let metadata = json!({ "semantic_version": "1.2.3",
                           "coverage": { "supported_targets": [LINUX] } });
    write(&root.join("versions/1.2.3.json"), &metadata.to_string());
    for pointer in ["latest_supported", "latest_validated"] {
        write(
            &root.join(format!("pointers/{pointer}/{LINUX}.txt")),
            "none\n",
        );
    }
    write(
        &root.join(format!("reports/1.2.3/coverage.{LINUX}.json")),
        &json!({ "deltas": {} }).to_string(),
    );
    let rules = json!({ "storage": { "committed": { "reports": {
        "depth_record": format!("{ROOT}/reports/<version>/depth-record.json") } } } });
    write(&root.join("RULES.json"), &rules.to_string());

    let evidence = temp.path().join(EVIDENCE);
    write(&evidence, "fn run() {}\n");
    let digest = file_sha256(&evidence).unwrap();
    let mut record = record;
    for result in record["execution"]["results"].as_array_mut().unwrap() {
        for file in result["evidence"].as_array_mut().unwrap() {
            file["sha256"] = json!(digest);
        }
    }
    write_record(temp.path(), &sealed_json(record, accept));
    temp
}

fn write_record(workspace: &Path, record: &Value) {
    let path = workspace.join(ROOT).join("reports/1.2.3/depth-record.json");
    write(&path, &record.to_string());
}

fn rows(workspace: &Path) -> Result<Vec<SupportRow>, String> {
    derive_rows_for_test_roots(workspace, &[("aider", ROOT)])
}

fn row<'a>(rows: &'a [SupportRow], target: &str) -> &'a SupportRow {
    rows.iter()
        .find(|row| row.version == "1.2.3" && row.target == target)
        .unwrap_or_else(|| panic!("no row for {target}"))
}

fn qualified(row: &SupportRow) -> bool {
    row.depth.as_ref().expect("depth-enrolled").capabilities[0].depth_qualified
}

#[test]
fn a_record_publishes_its_facts_and_the_rows_it_implies() {
    let temp = workspace(record(), true);
    let rows = rows(temp.path()).unwrap();
    assert_eq!(rows.len(), 2, "{rows:#?}");

    // Every result verified on linux, the evidence held and an entry made: the capability is
    // depth-qualified and the row stays `supported`.
    let linux = row(&rows, LINUX);
    assert!(qualified(linux));
    assert_eq!(linux.uaa_support, UaaSupportState::Supported);
    assert!(linux.evidence_notes.is_empty());

    // Darwin is not expected and has no metadata for it, and the record covers it, so it has a
    // row: unsupported in the manifest, no report, no pointer file, and insufficient depth.
    let darwin = row(&rows, DARWIN);
    assert_eq!(darwin.manifest_support, ManifestSupportState::Unsupported);
    assert_eq!(darwin.backend_support, BackendSupportState::Unsupported);
    assert_eq!(darwin.pointer_promotion, PointerPromotionState::None);
    let facts = darwin.depth.as_ref().unwrap();
    assert_eq!(facts.promises[0].outcomes, [Outcome::InsufficientDepth]);
    assert!(!qualified(darwin));

    // A pointer file for the target is read although the target is not expected.
    write(
        &temp
            .path()
            .join(ROOT)
            .join(format!("pointers/latest_validated/{DARWIN}.txt")),
        "1.2.3\n",
    );
    let rows = self::rows(temp.path()).unwrap();
    assert_eq!(
        row(&rows, DARWIN).pointer_promotion,
        PointerPromotionState::LatestValidated
    );

    // Without its metadata the version keeps the rows its record implies, unsupported in the
    // manifest on every target.
    fs::remove_file(temp.path().join(ROOT).join("versions/1.2.3.json")).unwrap();
    let rows = self::rows(temp.path()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        row(&rows, LINUX).manifest_support,
        ManifestSupportState::Unsupported
    );
    assert_eq!(row(&rows, LINUX).uaa_support, UaaSupportState::Unsupported);
}

#[test]
fn depth_facts_lower_a_supported_row_and_never_raise_one() {
    // No acceptance entry: the promise is not qualified, so the row is `partial`.
    let temp = workspace(record(), false);
    let rows = rows(temp.path()).unwrap();
    assert!(!qualified(row(&rows, LINUX)));
    assert_eq!(row(&rows, LINUX).uaa_support, UaaSupportState::Partial);

    // Evidence changed after acceptance: the result is `unverified`, the capability loses its
    // depth qualification, and the row falls to `partial`.
    let temp = workspace(record(), true);
    write(&temp.path().join(EVIDENCE), "fn run() { changed() }\n");
    let rows = self::rows(temp.path()).unwrap();
    let linux = row(&rows, LINUX);
    assert!(!qualified(linux));
    assert_eq!(linux.uaa_support, UaaSupportState::Partial);

    // A qualified capability over an unsupported backend leaves the row `unsupported`.
    let temp = workspace(record(), true);
    fs::remove_file(
        temp.path()
            .join(ROOT)
            .join(format!("reports/1.2.3/coverage.{LINUX}.json")),
    )
    .unwrap();
    let rows = self::rows(temp.path()).unwrap();
    assert!(qualified(row(&rows, LINUX)));
    assert_eq!(row(&rows, LINUX).uaa_support, UaaSupportState::Unsupported);
}

#[test]
fn publication_fails_on_a_record_it_cannot_read_or_that_its_root_does_not_adopt() {
    // A record that does not read is not published as no record.
    let temp = workspace(record(), true);
    write_record(temp.path(), &json!({}));
    let err = rows(temp.path()).unwrap_err();
    assert!(err.contains("depth-record.json"), "{err}");

    // A record of another agent's revision is refused by the reader too.
    let temp = workspace(record(), true);
    let mut other = sealed_json(record(), true);
    other["policy"]["selection"]["agent_id"] = json!("codex");
    write_record(temp.path(), &other);
    assert!(rows(temp.path()).is_err());

    // A root whose RULES.json names no record path holds none (record invariant 1), the case of
    // a root with no RULES.json at all included.
    let temp = workspace(record(), true);
    fs::remove_file(temp.path().join(ROOT).join("RULES.json")).unwrap();
    let err = rows(temp.path()).unwrap_err();
    assert!(err.contains("does not adopt Annex B"), "{err}");
    write(&temp.path().join(ROOT).join("RULES.json"), "{}");
    assert!(rows(temp.path())
        .unwrap_err()
        .contains("does not adopt Annex B"));
}
