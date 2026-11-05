//! The offline executor command against crafted bundles served by a live channel. The connected
//! chain with an authenticated population is in `isolated_maintenance/tests/ma4_evidence.rs`.
use super::*;
use crate::agent_registry::support_depth::ObligationId;
use crate::depth_record::{DepthResult, ObligationRef, ObligationResult};
use std::os::unix::fs::PermissionsExt;

const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const RECEIPT: &str =
    "cli_manifests/depth_fixture/reports/0.2.0/evidence/synthetic-wrapper-v1/1-1/R1/receipt.json";

type Edit = Box<dyn Fn(&mut Bundle)>;

pub(super) fn frozen() -> DepthRecord {
    crate::depth_ci::profile::Profile::Synthetic
        .record(include_str!(
            "../../../tests/fixtures/depth-synthetic/record.json"
        ))
        .unwrap()
}
/// A candidate holding only the frozen packet record.
pub(super) fn candidate() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(RECORD);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let raw = format!("{}\n", serde_json::to_string_pretty(&frozen()).unwrap());
    fs::write(path, raw).unwrap();
    dir
}
pub(super) fn crafted() -> Bundle {
    crafted_with(b"{\"claimed\":true}")
}
fn crafted_with(receipt: &[u8]) -> Bundle {
    let record = frozen();
    let receipt = receipt.to_vec();
    let reference = crate::depth_ci::reference(RECEIPT, &receipt);
    let mut execution = ExecutionPart {
        identity: String::new(),
        policy_identity: record.policy.identity.clone(),
        obligations_identity: record.obligations.as_ref().unwrap().identity.clone(),
        results: vec![ObligationResult {
            obligation: ObligationRef {
                operation: "run".into(),
                id: Some(ObligationId::R1),
                capability: None,
                target: "x86_64-unknown-linux-gnu".into(),
            },
            result: DepthResult::Verified,
            rationale: None,
            evidence: vec![reference.clone()],
        }],
        reuse_bindings: vec![],
        debt_grants: vec![],
        debt_retired: vec![],
    };
    execution.identity = crate::depth_record::part_identity(&execution).unwrap();
    Bundle {
        schema: SCHEMA.into(),
        invocation: invocation(Path::new("crafted")),
        record: RECORD.into(),
        agent_id: "depth_fixture".into(),
        version: "0.2.0".into(),
        event: record.event.clone(),
        policy_identity: record.policy.identity.clone(),
        obligations_identity: execution.obligations_identity.clone(),
        execution,
        files: vec![BundleFile {
            path: RECEIPT.into(),
            sha256: reference.sha256,
            hex: hex::encode(receipt),
        }],
    }
}
/// A host serving `bundle`, with this test's own process as both the host the assembler trusts
/// and the executor the host admits: in-process tests run both sides in one process.
pub(super) fn staged(candidate: &Path, bundle: &Bundle) -> Staged {
    let dir = private_dir().unwrap();
    let answer = publish(dir.path(), bundle);
    in_process(serve(dir, candidate, answer).unwrap())
}
pub(super) fn in_process(staged: Staged) -> Staged {
    peer::trust_own_process();
    staged.channel().admission.admit(std::process::id());
    staged
}
/// Every file under `root`, with its bytes.
pub(super) fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            if meta.is_dir() {
                stack.push(path);
            } else {
                files.insert(path.clone(), fs::read(&path).unwrap_or_default());
            }
        }
    }
    files
}
fn refused(root: &Path, bundle: &Bundle) -> String {
    let before = snapshot(root);
    let staged = staged(root, bundle);
    let error = assemble(root, Some(staged.locator().to_owned())).expect_err("refused");
    assert_eq!(snapshot(root), before, "a refusal wrote: {error}");
    error
}

#[test]
fn writes_only_e_and_its_files_through_the_live_channel() {
    let root = candidate();
    let bundle = crafted();
    let staged = staged(root.path(), &bundle);
    let summary = assemble(root.path(), Some(staged.locator().to_owned())).unwrap();
    assert!(summary.contains("1 obligations"), "{summary}");
    let record = DepthRecord::parse(
        &fs::read_to_string(root.path().join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    assert_eq!(record.execution.as_ref(), Some(&bundle.execution));
    let frozen = frozen();
    assert_eq!(
        (&record.event, &record.policy, &record.obligations),
        (&frozen.event, &frozen.policy, &frozen.obligations)
    );
    assert_eq!(record.acceptance_entries, frozen.acceptance_entries);
    let files: Vec<_> = snapshot(root.path()).into_keys().collect();
    assert_eq!(
        files,
        [RECORD, RECEIPT]
            .map(|p| root.path().join(p))
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    );
    // Running again under the same invocation is idempotent.
    assemble(root.path(), Some(staged.locator().to_owned())).unwrap();
}

#[test]
fn only_the_live_invocation_for_this_candidate_counts_as_a_route() {
    let root = candidate();
    let before = snapshot(root.path());
    // Production and a standalone run in the candidate have no channel.
    let error = assemble(root.path(), None).unwrap_err();
    assert!(error.contains("no invocation channel"), "{error}");
    let staged = staged(root.path(), &crafted());
    let locator = staged.locator().to_owned();
    let offer = channel::request(&locator, &fs::canonicalize(root.path()).unwrap()).unwrap();
    // Another root, even a copy of the candidate, is not the candidate this invocation serves.
    let other = candidate();
    let error = assemble(other.path(), Some(locator.clone())).unwrap_err();
    assert!(error.contains("no evidence for that root"), "{error}");
    // A copied bundle or any other file named as the channel is not a live invocation.
    let copy = other.path().join("bundle.json");
    fs::copy(&offer.bundle, &copy).unwrap();
    for path in [copy.clone(), other.path().join("absent")] {
        let error = assemble(root.path(), Some(path)).unwrap_err();
        assert!(error.contains("not live"), "{error}");
    }
    // Once the invocation ends, its channel and bundle are gone and nothing replays it.
    drop(staged);
    assert!(!offer.bundle.exists());
    assert!(!locator.parent().unwrap().exists());
    let error = assemble(root.path(), Some(locator)).unwrap_err();
    assert!(error.contains("not live"), "{error}");
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn substituted_or_replayed_bundles_refuse() {
    let root = candidate();
    let before = snapshot(root.path());
    // The earlier invocation staged other (still self-consistent) evidence.
    let earlier = staged(root.path(), &crafted_with(b"{\"earlier\":true}"));
    let canonical = fs::canonicalize(root.path()).unwrap();
    let old = channel::request(earlier.locator(), &canonical).unwrap();
    let current = staged(root.path(), &crafted());
    let offer = channel::request(current.locator(), &canonical).unwrap();
    let dir = offer.bundle.parent().unwrap();
    fs::set_permissions(&offer.bundle, fs::Permissions::from_mode(0o600)).unwrap();
    // An earlier invocation's bundle, a re-signed copy of it, and a symlink to it all refuse.
    let mut resigned: Bundle = serde_json::from_slice(&fs::read(&old.bundle).unwrap()).unwrap();
    resigned.invocation = offer.invocation.clone();
    for (substitute, expected) in [
        (
            fs::read(&old.bundle).unwrap(),
            "differs from what this invocation",
        ),
        (
            serde_json::to_vec(&resigned).unwrap(),
            "differs from what this invocation",
        ),
    ] {
        fs::write(&offer.bundle, substitute).unwrap();
        let error = assemble(root.path(), Some(current.locator().to_owned())).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    fs::remove_file(&offer.bundle).unwrap();
    std::os::unix::fs::symlink(&old.bundle, dir.join("bundle.json")).unwrap();
    let error = assemble(root.path(), Some(current.locator().to_owned())).unwrap_err();
    assert!(error.contains("not a bounded regular file"), "{error}");
    assert_eq!(snapshot(root.path()), before);
}

#[test]
fn traversal_symlinks_forged_files_and_foreign_destinations_refuse() {
    let edits: Vec<(Edit, &str)> = vec![
        (
            Box::new(|b| b.files[0].path = format!("{RECEIPT}/../../../x")),
            "unsafe repository-relative",
        ),
        (
            Box::new(|b| b.files[0].path = "docs/specs/forged.md".into()),
            "outside the reporting destination",
        ),
        (
            Box::new(|b| b.record = "docs/agents/lifecycle/maintenance-request.toml".into()),
            "not a normal reporting destination",
        ),
        (
            Box::new(|b| b.files[0].hex = hex::encode(b"forged")),
            "digest mismatch",
        ),
        (Box::new(|b| b.files.clear()), "does not carry"),
        (
            Box::new(|b| b.execution.debt_retired = vec!["forged".into()]),
            "not bound to the staged P/O",
        ),
    ];
    for (edit, expected) in edits {
        let root = candidate();
        let mut bundle = crafted();
        edit(&mut bundle);
        let error = refused(root.path(), &bundle);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    // A symlinked reporting directory never redirects a write.
    let root = candidate();
    let outside = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(
        outside.path(),
        root.path()
            .join("cli_manifests/depth_fixture/reports/0.2.0/evidence"),
    )
    .unwrap();
    let error = refused(root.path(), &crafted());
    assert!(error.contains("symlink"), "{error}");
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}

#[test]
fn another_generation_p_o_or_target_refuses() {
    let rebind = |b: &mut Bundle, policy: &str| {
        b.policy_identity = policy.into();
        b.execution.policy_identity = policy.into();
        b.execution.identity = crate::depth_record::part_identity(&b.execution).unwrap();
    };
    let edits: Vec<(Edit, &str)> = vec![
        (
            Box::new(|b| {
                if let Event::Maintenance { request_commit, .. } = &mut b.event {
                    *request_commit = "1".repeat(40);
                }
            }),
            "another generation",
        ),
        (
            Box::new(move |b| rebind(b, &"0".repeat(64))),
            "another frozen P/O",
        ),
        (
            Box::new(|b| {
                b.version = "0.3.0".into();
                b.record = RECORD.replace("0.2.0", "0.3.0");
            }),
            "outside the reporting destination",
        ),
    ];
    for (edit, expected) in edits {
        let root = candidate();
        let mut bundle = crafted();
        edit(&mut bundle);
        let error = refused(root.path(), &bundle);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    // A host whose authentication failed serves only its refusal, never evidence.
    let root = candidate();
    let before = snapshot(root.path());
    let target = Target {
        agent_id: "depth_fixture".into(),
        version: "0.2.0".into(),
        manifest_root: "cli_manifests/depth_fixture".into(),
    };
    let staged =
        in_process(stage_answer(root.path(), Err("missing adoption".into()), &target).unwrap());
    let error = assemble(root.path(), Some(staged.locator().to_owned())).unwrap_err();
    assert!(
        error.contains("no authenticated evidence was staged")
            && error.contains("missing adoption"),
        "{error}"
    );
    assert_eq!(snapshot(root.path()), before);
}
