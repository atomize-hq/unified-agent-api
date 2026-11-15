//! OR-P13 credential-free two-phase composition using the real A checkers.
#![cfg(target_os = "linux")]
use super::*;
use crate::{
    depth_evidence::{EvidenceVerdicts, SyntheticHost},
    readonly_gates as gates,
};

mod checkers;
mod drift;
mod facts;
mod process;
mod standalone;
mod validator;

const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const PRODUCTION_REFUSAL: &str =
    "agent `depth_fixture` carries `support_depth`, which no command enforces yet";

fn setup() -> (Fixture, SyntheticHost) {
    let (f, host) = super::ma5_closeout::publication_ready();
    {
        let _route = f.install();
        let markdown =
            crate::capability_matrix::generate_markdown_for_workspace(f.root.path()).unwrap();
        write(
            f.root.path(),
            crate::capability_matrix::DEFAULT_OUT_PATH,
            markdown,
        );
    }
    if !git(f.root.path(), &["status", "--porcelain=v1"]).is_empty() {
        super::ma5_closeout::advance(&f, &host);
    }
    (f, host)
}

fn proposed_files(f: &Fixture) -> BTreeMap<String, Vec<u8>> {
    let root = f.root.path();
    super::ma5_closeout::prepare(root).unwrap();
    let changed = git(root, &["diff", "HEAD", "--name-only"]);
    let added = git(root, &["ls-files", "--others", "--exclude-standard"]);
    let files = changed
        .lines()
        .chain(added.lines())
        .map(|p| (p.to_owned(), fs::read(root.join(p)).unwrap()))
        .collect();
    // Restore only the test-owned temporary caller after collecting the real writer's outputs.
    git(root, &["reset", "--hard", "HEAD"]);
    for path in added.lines() {
        fs::remove_file(root.join(path)).unwrap();
    }
    files
}

fn materialize(root: &Path, files: &BTreeMap<String, Vec<u8>>) {
    for (path, bytes) in files {
        write(root, path, bytes);
    }
}

fn standalone(root: &Path) {
    crate::support_matrix::check_in_workspace(root).unwrap();
    crate::capability_matrix::check_in_workspace(root).unwrap();
    crate::capability_publication::check_in_workspace(root).unwrap();
    crate::runtime_advertising::check_in_workspace(root).unwrap();
    gates::with_committed_evaluation(root, || gates::check_shared(root)).unwrap();
}

#[test]
fn readonly_gates_two_phases_real_checkers_and_independent_process() {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    let files = proposed_files(&f);
    let before = super::ma5_closeout::snapshot(root);
    {
        let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
        let proposal = captured.proposal(root).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let view = proposal.isolated_view(root, scratch.path()).unwrap();
        materialize(scratch.path(), &files);
        let _binding = captured.bind_view(&view).unwrap();
        assert!(!scratch.path().join(".git").exists());
        EvidenceVerdicts::with_publication_view(
            root,
            scratch.path(),
            &proposal,
            &captured,
            &view,
            || {
                gates::check_shared(scratch.path())?;
                checkers::refusals(scratch.path(), captured.registry());
                // Exact record sealing includes entries even though stated_sha256 excludes them.
                let original = fs::read(scratch.path().join(RECORD)).unwrap();
                let mut changed: Value = serde_json::from_slice(&original).unwrap();
                changed["acceptance_entries"] = json!([]);
                write(
                    scratch.path(),
                    RECORD,
                    serde_json::to_vec(&changed).unwrap(),
                );
                assert_eq!(
                    crate::support_matrix::check_publication(scratch.path()).unwrap_err(),
                    "publication record changed after evaluation"
                );
                write(scratch.path(), RECORD, original);
                gates::check_shared(scratch.path())
            },
        )
        .unwrap();
        assert_eq!(super::ma5_closeout::snapshot(root), before);
    }
    materialize(root, &files);
    super::ma5_closeout::advance(&f, &host);
    let committed = super::ma5_closeout::snapshot(root);
    let admissions = f.api.3.facts.lock().unwrap().clone();
    standalone(root);
    super::publication::assert_qualified(root);
    process::evaluate(&f, &host);
    assert_eq!(super::ma5_closeout::snapshot(root), committed);
    assert_eq!(*f.api.3.facts.lock().unwrap(), admissions);

    // A semantically harmless candidate change receives a new independent evaluation.
    write(root, "docs/input.txt", "harmless candidate documentation\n");
    super::ma5_closeout::advance(&f, &host);
    standalone(root);

    // The earlier proposal pass supplies no authority to an altered evidence binding.
    let record = super::ma5_closeout::record(root);
    let receipt = &record.execution.as_ref().unwrap().results[0].evidence[0].path;
    let mut bytes = fs::read(root.join(receipt)).unwrap();
    bytes.push(b'\n');
    write(root, receipt, bytes);
    super::ma5_closeout::advance(&f, &host);
    let changed = super::ma5_closeout::snapshot(root);
    EvidenceVerdicts::take_publication_evaluations();
    let error = crate::support_matrix::check_in_workspace(root).unwrap_err();
    assert!(
        error.ends_with("is stale; regenerate with `cargo run -p xtask -- support-matrix`"),
        "{error}"
    );
    assert!(!EvidenceVerdicts::take_publication_evaluations().is_empty());
    assert_eq!(super::ma5_closeout::snapshot(root), changed);
}

#[test]
fn readonly_gates_generation_and_neutral_writers_keep_their_controls() {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    super::ma5_closeout::prepare(root).unwrap();
    super::ma5_closeout::advance(&f, &host);
    let before = super::ma5_closeout::snapshot(root);
    EvidenceVerdicts::take_publication_evaluations();
    let generated = crate::support_matrix::generate_publication_artifacts(root).unwrap();
    let artifact: crate::support_matrix::SupportMatrixArtifact =
        serde_json::from_str(&generated.json).unwrap();
    let depth = artifact
        .rows
        .iter()
        .find(|r| r.agent == "depth_fixture" && r.version == "0.2.0")
        .unwrap()
        .depth
        .as_ref()
        .unwrap();
    facts::assert_unverified(depth);
    assert!(EvidenceVerdicts::take_publication_evaluations().is_empty());
    assert_eq!(
        crate::runtime_advertising::run_in_workspace(
            root,
            crate::runtime_advertising::Args { check: false }
        )
        .unwrap_err()
        .to_string(),
        "runtime-advertising-model: unsupported depth write (declared enrollment)"
    );
    assert_eq!(
        crate::support_matrix::write_publication_artifacts(root, &generated).unwrap_err(),
        "support-matrix writer: unsupported depth write (declared enrollment)"
    );
    assert_eq!(super::ma5_closeout::snapshot(root), before);
    drop(_route);
    assert_eq!(
        registry(root),
        Err(RegistryError::Production(PRODUCTION_REFUSAL.into()))
    );
    assert_eq!(
        crate::runtime_advertising::check_in_workspace(root).unwrap_err(),
        PRODUCTION_REFUSAL
    );
    assert_eq!(
        crate::runtime_advertising::revision::Revision::workspace(root)
            .err()
            .unwrap(),
        PRODUCTION_REFUSAL
    );
    assert_eq!(super::ma5_closeout::snapshot(root), before);
}

#[test]
fn readonly_gates_closing_observations_refuse_host_and_parent_drift() {
    let (f, host) = setup();
    let root = f.root.path();
    let _route = f.install();
    let _reader = host.install();
    for case in ["host", "transport", "protected_parent", "parent"] {
        let mut concurrent = None;
        let error = gates::with_committed_evaluation(root, || {
            gates::check_shared(root)?;
            match case {
                "host" => f.api.put(
                    "repos/fixture/isolated",
                    json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
                ),
                "transport" => f.api.4.store(true, std::sync::atomic::Ordering::SeqCst),
                "protected_parent" => f.api.put(
                    "repos/fixture/isolated/branches/staging",
                    json!({"name":"staging","protected":true,"commit":{"sha":f.baseline}}),
                ),
                "parent" => {
                    git(
                        root,
                        &["commit", "--allow-empty", "-m", "concurrent candidate"],
                    );
                }
                _ => unreachable!(),
            }
            concurrent = Some(super::ma5_closeout::snapshot(root));
            Ok(())
        })
        .unwrap_err();
        let expected = match case {
            "host" => "isolated repository id, owner id or full name differs",
            "transport" => "one-shot isolated host transport unavailable",
            "protected_parent" => "isolated host identity, source or protected tip changed",
            "parent" => "isolated caller workspace or Git ownership changed",
            _ => unreachable!(),
        };
        assert_eq!(error, expected);
        assert_eq!(super::ma5_closeout::snapshot(root), concurrent.unwrap());
        f.host();
        host.staging(&git(root, &["rev-parse", "HEAD"]));
    }
    write(root, "producer/source.rs", "changed producer\n");
    super::ma5_closeout::advance(&f, &host);
    let before = super::ma5_closeout::snapshot(root);
    assert_eq!(
        crate::support_matrix::check_in_workspace(root).unwrap_err(),
        "executed/local/protected producer source differs: producer/source.rs"
    );
    assert_eq!(super::ma5_closeout::snapshot(root), before);
}
