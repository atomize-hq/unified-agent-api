//! CP-P3 option A: publication belongs to the exact owning post-entry batch.
#![cfg(target_os = "linux")]
use super::*;
pub(super) fn assert_qualified(root: &Path) {
    let artifact: crate::support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let row = artifact
        .rows
        .iter()
        .find(|r| {
            r.agent == "depth_fixture"
                && r.version == "0.2.0"
                && r.target == "x86_64-unknown-linux-gnu"
        })
        .unwrap();
    let depth = row.depth.as_ref().unwrap();
    assert!(!depth.promises.is_empty());
    assert!(depth.promises.iter().all(|p| p.qualified
        && p.obligations
            .iter()
            .all(|o| o.result == crate::depth_record::DepthResult::Verified)));
}

#[test]
fn publication_closeout_writer_socket_free_qualified_batch() {
    use super::ma5_closeout as ma5;
    let (f, host) = ma5::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    ma5::author(&f, &host);
    let parent = git(root, &["rev-parse", "HEAD"]);
    let before = ma5::record(root);
    ma5::close(root).unwrap();
    assert!(before.acceptance_entries.is_empty());
    assert!(matches!(
        ma5::record(root).acceptance_entries.as_slice(),
        [crate::depth_record::AcceptanceEntry::MaintenanceCloseout { .. }]
    ));
    assert_qualified(root);
    ma5::advance(&f, &host);
    crate::depth_acceptance::check_integration(root, &parent).unwrap();
}

/// Exercise the public command without an externally supplied verdict scope.
pub(super) fn validate_candidate(root: &Path, parent: &str) -> i32 {
    crate::manifest_validate::run(crate::manifest_validate::Args {
        root: Some(root.join("cli_manifests/depth_fixture")),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Check,
        json: true,
        tip: Some(parent.into()),
    })
}

#[test]
fn publication_default_keeps_production_verdicts_with_a_route_and_in_scratch() {
    use crate::support_matrix as matrix;
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    super::ma5_closeout::prepare(root).unwrap();
    assert_qualified(root);
    super::ma5_closeout::advance(&f, &host);
    let before = super::ma5_closeout::snapshot(root);
    let ordinary = matrix::generate_publication_artifacts(root).unwrap();
    let rows: matrix::SupportMatrixArtifact = serde_json::from_str(&ordinary.json).unwrap();
    let depth = rows
        .rows
        .iter()
        .find(|r| r.agent == "depth_fixture" && r.version == "0.2.0")
        .unwrap()
        .depth
        .as_ref()
        .unwrap();
    assert!(depth.promises.iter().all(|p| !p.qualified
        && p.obligations
            .iter()
            .all(|o| o.result == crate::depth_record::DepthResult::Unverified)));

    // Without --tip the public validator still uses production verdicts, so qualified
    // synthetic publication disagrees without triggering an authenticated evaluation.
    crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations();
    assert_eq!(
        crate::manifest_validate::run(crate::manifest_validate::Args {
            root: Some(root.join("cli_manifests/depth_fixture")),
            rules: None,
            schema: None,
            version_schema: None,
            mode: crate::manifest_validate::Mode::Check,
            json: true,
            tip: None,
        }),
        2
    );
    assert!(crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations().is_empty());

    // This view has no Git repository or remote of its own. Borrowing registry/Git inputs
    // must suffice for ordinary rendering, without triggering isolated evidence observation.
    let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    let _binding = captured.bind_view(&view).unwrap();
    assert!(!scratch.path().join(".git").exists());
    let rendered = matrix::generate_publication_artifacts(scratch.path()).unwrap();
    assert_eq!(rendered.json, ordinary.json);
    assert_eq!(rendered.markdown, ordinary.markdown);
    assert_eq!(rendered.runtime_support_data, ordinary.runtime_support_data);
    assert_eq!(super::ma5_closeout::snapshot(root), before);
}

#[test]
fn publication_ma5_socket_free_own_rows_and_in1_closeout_attribution() {
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let before: crate::support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let runtime =
        fs::read(root.join(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH))
            .unwrap();
    let capabilities = fs::read(root.join(crate::capability_matrix::DEFAULT_OUT_PATH)).unwrap();
    let parent = git(root, &["rev-parse", "HEAD"]);
    super::ma5_closeout::prepare(root).unwrap();
    assert_qualified(root);
    let after: crate::support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        before
            .rows
            .iter()
            .filter(|r| r.agent != "depth_fixture" || r.version != "0.2.0")
            .collect::<Vec<_>>(),
        after
            .rows
            .iter()
            .filter(|r| r.agent != "depth_fixture" || r.version != "0.2.0")
            .collect::<Vec<_>>()
    );
    assert_eq!(
        fs::read(root.join(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH))
            .unwrap(),
        runtime
    );
    assert_eq!(
        fs::read(root.join(crate::capability_matrix::DEFAULT_OUT_PATH)).unwrap(),
        capabilities
    );
    assert_eq!(
        super::ma5_closeout::record(root).acceptance_entries.len(),
        1
    );
    let head = candidate(&f, &parent);
    crate::depth_acceptance::check_integration(root, &parent).unwrap();
    crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations();
    assert_eq!(validate_candidate(root, &parent), 0);
    assert!(!crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations().is_empty());
    f.host();
    host.staging(&head);
    git(root, &["update-ref", "refs/remotes/origin/staging", &head]);
    crate::depth_acceptance::check_integration(root, &parent).unwrap();
    crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations();
    assert_eq!(validate_candidate(root, &parent), 0);
    assert!(!crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations().is_empty());
}

#[test]
fn publication_retained_closeout_and_appended_duplicate_cannot_own_new_qualification() {
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let before: crate::support_matrix::SupportMatrixArtifact = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let runtime =
        fs::read(root.join(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH))
            .unwrap();
    super::ma5_closeout::prepare(root).unwrap();
    super::ma5_closeout::advance(&f, &host);
    // A retained closeout alone cannot own a *new* qualification in a later candidate.
    let stale = crate::support_matrix::render_publication_artifacts(
        &fs::read_to_string(root.join(crate::support_matrix::MARKDOWN_OUTPUT_PATH)).unwrap(),
        &before.rows,
        String::from_utf8(runtime).unwrap(),
    )
    .unwrap();
    let qualified_json = fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap();
    let qualified_markdown =
        fs::read(root.join(crate::support_matrix::MARKDOWN_OUTPUT_PATH)).unwrap();
    write(root, crate::support_matrix::JSON_OUTPUT_PATH, stale.json);
    write(
        root,
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        stale.markdown,
    );
    let stale_tip = super::ma5_closeout::advance(&f, &host);
    write(
        root,
        crate::support_matrix::JSON_OUTPUT_PATH,
        &qualified_json,
    );
    write(
        root,
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        &qualified_markdown,
    );
    candidate(&f, &stale_tip);
    let snapshot = super::ma5_closeout::snapshot(root);
    let refusal = crate::depth_acceptance::check_integration(root, &stale_tip).unwrap_err();
    assert!(refusal.contains("promotion"), "{refusal}");
    assert_eq!(super::ma5_closeout::snapshot(root), snapshot);

    // Appending an identical retained entry does not make it a new owning closeout.
    git(root, &["reset", "--hard", &stale_tip]);
    let mut record = super::ma5_closeout::record(root);
    record
        .acceptance_entries
        .push(record.acceptance_entries[0].clone());
    write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
    write(
        root,
        crate::support_matrix::JSON_OUTPUT_PATH,
        &qualified_json,
    );
    write(
        root,
        crate::support_matrix::MARKDOWN_OUTPUT_PATH,
        &qualified_markdown,
    );
    candidate(&f, &stale_tip);
    let snapshot = super::ma5_closeout::snapshot(root);
    let refusal = crate::depth_acceptance::check_integration(root, &stale_tip).unwrap_err();
    assert!(refusal.contains("promotion"), "{refusal}");
    assert_eq!(super::ma5_closeout::snapshot(root), snapshot);
}

fn candidate(f: &Fixture, parent: &str) -> String {
    let root = f.root.path();
    git(root, &["add", "."]);
    git(root, &["commit", "-m", "qualified publication candidate"]);
    let head = git(root, &["rev-parse", "HEAD"]);
    for base in [parent, &f.baseline] {
        f.api.put(
            &format!("repos/fixture/isolated/compare/{base}...{head}"),
            json!({"status":"ahead","merge_base_commit":{"sha":base}}),
        );
    }
    f.api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    head
}

pub(super) fn assemble(f: &Fixture) {
    let root = f.root.path();
    let context = f.resolve().unwrap();
    let evidence_host = context.evidence_host(root).unwrap();
    let assembly =
        crate::depth_evidence::EvidenceVerdicts::assemble_view_fixture(root, &evidence_host)
            .unwrap();
    let mut record = super::ma5_closeout::record(root);
    record.execution = Some(assembly.execution);
    for (reference, bytes) in assembly.files {
        write(root, &reference.path, bytes);
    }
    write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
}
pub(super) fn refresh(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) {
    let _installed = f.install();
    let _reader = host.install();
    refresh_installed(f, host);
}
/// Refresh inside a chain whose route and evidence reader are already installed.
pub(super) fn refresh_installed(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) {
    let root = f.root.path();
    let bundle = crate::support_matrix::generate_publication_artifacts(root).unwrap();
    let mut changed = false;
    for (path, bytes) in [
        (crate::support_matrix::JSON_OUTPUT_PATH, bundle.json),
        (crate::support_matrix::MARKDOWN_OUTPUT_PATH, bundle.markdown),
        (
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            bundle.runtime_support_data,
        ),
    ] {
        if fs::read(root.join(path)).unwrap() != bytes.as_bytes() {
            write(root, path, bytes);
            changed = true;
        }
    }
    if changed {
        f.commit();
        host.staging(&git(root, &["rev-parse", "HEAD"]));
    }
}
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

#[test]
fn publication_ma5_refuses_tampered_own_foreign_and_runtime_rows_without_writes() {
    use crate::support_matrix as matrix;
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let parent = git(root, &["rev-parse", "HEAD"]);
    super::ma5_closeout::prepare(root).unwrap();
    let original = [
        CLOSEOUT,
        matrix::JSON_OUTPUT_PATH,
        matrix::MARKDOWN_OUTPUT_PATH,
        matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
    ]
    .into_iter()
    .map(|p| (PathBuf::from(p), fs::read(root.join(p)).unwrap()))
    .collect::<Vec<_>>();
    git(root, &["reset", "--hard", &parent]);
    git(root, &["clean", "-fd"]);
    let proposal = crate::isolated_maintenance::route(root)
        .unwrap()
        .unwrap()
        .proposal(root)
        .unwrap();
    let before = super::ma5_closeout::snapshot(root);
    for tamper in 0..5 {
        let mut files = original.clone();
        if tamper < 3 {
            let mut artifact: matrix::SupportMatrixArtifact =
                serde_json::from_slice(&files[1].1).unwrap();
            let row = artifact
                .rows
                .iter_mut()
                .find(|r| match tamper {
                    0 => r.agent == "depth_fixture" && r.version == "0.2.0",
                    1 => r.agent != "depth_fixture",
                    _ => r.agent == "depth_fixture" && r.version != "0.2.0",
                })
                .expect("fixture must include own, foreign-agent and foreign-version rows");
            if tamper == 0 {
                row.depth.as_mut().unwrap().promises[0].qualified = false;
            } else {
                row.evidence_notes.push("foreign edit".into());
            }
            // Coherent Markdown cannot launder a foreign row or a false own depth fact.
            let bundle = matrix::render_publication_artifacts(
                std::str::from_utf8(&files[2].1).unwrap(),
                &artifact.rows,
                String::from_utf8(files[3].1.clone()).unwrap(),
            )
            .unwrap();
            files[1].1 = bundle.json.into_bytes();
            files[2].1 = bundle.markdown.into_bytes();
        } else if tamper == 3 {
            files[3].1.push(b'\n');
        } else {
            files.push((
                crate::capability_matrix::DEFAULT_OUT_PATH.into(),
                b"tampered capability publication".to_vec(),
            ));
        }
        let err = proposal.plan(root, files).unwrap_err();
        assert!(
            err.contains("foreign rows")
                || err.contains("runtime or capability")
                || err.contains("unowned output"),
            "{err}"
        );
        assert_eq!(super::ma5_closeout::snapshot(root), before);
    }
}
const CLOSEOUT: &str =
    "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json";

#[test]
fn publication_ma6_renders_and_admits_its_post_entry_record() {
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    super::ma5_closeout::prepare(root).unwrap();
    let parent = git(root, &["rev-parse", "HEAD"]);
    let closed = candidate(&f, &parent);
    crate::depth_acceptance::check_integration(root, &parent).unwrap();
    f.host();
    host.staging(&closed);
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &closed],
    );
    let args = crate::manifest_promotion::Args {
        agent: "depth_fixture".into(),
        version: "0.2.0".into(),
        passed_targets: vec!["x86_64-unknown-linux-gnu".into()],
        validation_commit: closed.clone(),
        write: false,
    };
    let before = super::ma5_closeout::snapshot(root);
    crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations();
    let plan = crate::manifest_promotion::build_plan(root, &args).unwrap();
    let evaluations = crate::depth_evidence::EvidenceVerdicts::take_publication_evaluations();
    assert!(!evaluations.is_empty());
    assert!(evaluations.iter().all(|entries| matches!(
        entries.as_slice(),
        [
            crate::depth_record::AcceptanceEntry::MaintenanceCloseout { .. },
            crate::depth_record::AcceptanceEntry::Promotion { .. }
        ]
    )));
    assert_eq!(super::ma5_closeout::snapshot(root), before);
    plan.apply(root).unwrap();
    assert_qualified(root);
    assert!(matches!(
        super::ma5_closeout::record(root)
            .acceptance_entries
            .as_slice(),
        [
            crate::depth_record::AcceptanceEntry::MaintenanceCloseout { .. },
            crate::depth_record::AcceptanceEntry::Promotion { .. }
        ]
    ));
    let promoted = candidate(&f, &closed);
    crate::depth_acceptance::check_integration(root, &closed).unwrap();
    assert_eq!(validate_candidate(root, &closed), 0);
    f.host();
    host.staging(&promoted);
    git(
        root,
        &["update-ref", "refs/remotes/origin/staging", &promoted],
    );
    crate::depth_acceptance::check_integration(root, &closed).unwrap();
    assert_eq!(validate_candidate(root, &closed), 0);
}

#[test]
fn publication_ma5_preserves_ordinary_target_outside_depth_selection() {
    use crate::support_matrix as matrix;
    let (f, host) = super::ma5_closeout::publication_ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    let ordinary = "aarch64-unknown-linux-gnu";
    let current = "cli_manifests/depth_fixture/current.json";
    let mut value: Value = serde_json::from_slice(&fs::read(root.join(current)).unwrap()).unwrap();
    value["expected_targets"]
        .as_array_mut()
        .unwrap()
        .push(json!(ordinary));
    write(root, current, serde_json::to_vec_pretty(&value).unwrap());
    let layout =
        crate::root_intake_layout::RootIntakeLayout::new(root.join("cli_manifests/depth_fixture"));
    for path in [
        layout.latest_supported_pointer_path(ordinary),
        layout.latest_validated_pointer_path(ordinary),
    ] {
        write(
            root,
            path.strip_prefix(root).unwrap().to_str().unwrap(),
            "none\n",
        );
    }
    let mut metadata: Value = serde_json::from_slice(
        &fs::read(root.join("cli_manifests/depth_fixture/versions/0.1.0.json")).unwrap(),
    )
    .unwrap();
    metadata["semantic_version"] = json!("0.2.0");
    metadata["coverage"]["supported_targets"]
        .as_array_mut()
        .unwrap()
        .push(json!(ordinary));
    write(
        root,
        "cli_manifests/depth_fixture/versions/0.2.0.json",
        serde_json::to_vec_pretty(&metadata).unwrap(),
    );
    super::ma5_closeout::advance(&f, &host);
    refresh_installed(&f, &host);
    git(
        root,
        &[
            "update-ref",
            "refs/remotes/origin/staging",
            &git(root, &["rev-parse", "HEAD"]),
        ],
    );
    let ordinary_row = || {
        let artifact: matrix::SupportMatrixArtifact =
            serde_json::from_slice(&fs::read(root.join(matrix::JSON_OUTPUT_PATH)).unwrap())
                .unwrap();
        artifact
            .rows
            .into_iter()
            .find(|r| r.agent == "depth_fixture" && r.version == "0.2.0" && r.target == ordinary)
            .unwrap()
    };
    let before = ordinary_row();
    assert!(before.depth.is_none());
    super::ma5_closeout::prepare(root).unwrap();
    assert_qualified(root);
    assert_eq!(ordinary_row(), before);
    assert!(ordinary_row().depth.is_none());
}
