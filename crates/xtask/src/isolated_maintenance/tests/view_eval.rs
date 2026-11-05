//! B.7a: borrowed authority over a proposed post-entry scratch view.
#![cfg(target_os = "linux")]
use super::*;
use super::{
    ma4_route::run_as,
    prepare::{acquire, workspace},
};
use crate::{
    agent_lifecycle::maintenance_request_path,
    agent_maintenance::{
        execute,
        prepare::{apply_prepare_plan, build_prepare_plan},
        prepare_closeout, stand_down,
    },
    depth_ci::profile::Profile,
    depth_record::{AcceptanceEntry, DepthRecord, DepthResult},
};
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json";
const ENTRY: &str = "isolated_maintenance::tests::ma4_evidence::executor_entry";
fn set_local(root: &Path, tip: &str) {
    git(root, &["update-ref", "refs/remotes/origin/staging", tip]);
}
fn executor(dir: &Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("executor.sh");
    let exe = dir.join("assembler");
    std::fs::copy(std::env::current_exe().unwrap(), &exe).unwrap();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprompt=$(cat)\ncase \"$prompt\" in\n*'Repository preflight'*) echo {} ;;\n*) \"{}\" --exact {ENTRY} --ignored --quiet || exit 1 ;;\nesac\n",
            execute::PREFLIGHT_SENTINEL,
            exe.display(),
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_string_lossy().into_owned()
}

fn frozen(classified: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
    let (f, mut args) = workspace();
    let root = f.root.path();
    args.request_commit = "0b1e1f0a478af0cd06d6ba6998ab594f09fbe592".into();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"),
        root.join(".gitignore"),
    )
    .unwrap();
    f.commit();
    {
        let _installed = f.install();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
        acquire(root, "0.2.0");
        f.commit();
        let frozen = build_prepare_plan(root, &args).unwrap();
        apply_prepare_plan(root, &frozen).unwrap();
        f.commit();
        if classified {
            // The approved synthetic CORE classification (B.3) enters as a new generation.
            let [(_, basis)] = frozen.classification_bases.as_slice() else {
                panic!("one basis: {:?}", frozen.classification_bases)
            };
            let record = DepthRecord::parse(
                &std::fs::read_to_string(root.join(RECORD)).unwrap(),
                "depth_fixture",
                "0.2.0",
            )
            .unwrap();
            let surfaces: Vec<_> = record
                .obligations
                .unwrap()
                .classification_required
                .into_iter()
                .map(|c| c.surface)
                .collect();
            classify(&f, &surfaces, basis);
            f.commit();
            args.request_recorded_at = "2026-11-04T00:00:00Z".into();
            args.request_commit = "0c2e2f1b589bf1de17e7c7c7ab695f10cef6fa03".into();
            apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
            f.commit();
        }
    }
    write(
        root,
        Profile::Synthetic.fixture(),
        std::fs::read(root.join(RECORD)).unwrap(),
    );
    let host = crate::depth_evidence::synthetic_host(root);
    f.host();
    (f, host)
}

fn delivered(classified: bool, channel: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
    let (f, host) = frozen(classified);
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let binary = executor(bin.path());
    {
        let _installed = f.install();
        let _reader = host.install();
        let _gates = execute::gate_stub::stub();
        if channel {
            run_as(root, &binary, "authority").unwrap();
        } else {
            let context = f.resolve().unwrap();
            let host = context.evidence_host(root).unwrap();
            let assembly =
                crate::depth_evidence::EvidenceVerdicts::assemble_view_fixture(root, &host)
                    .unwrap();
            let mut record = DepthRecord::parse(
                &fs::read_to_string(root.join(RECORD)).unwrap(),
                "depth_fixture",
                "0.2.0",
            )
            .unwrap();
            record.execution = Some(assembly.execution);
            for (reference, bytes) in assembly.files {
                write(root, &reference.path, bytes);
            }
            write(root, RECORD, serde_json::to_vec_pretty(&record).unwrap());
        }
    }
    f.commit();
    host.staging(&git(root, &["rev-parse", "HEAD"]));
    (f, host)
}

fn classify(
    f: &Fixture,
    surfaces: &[crate::agent_maintenance::support_audit::SurfaceIdentity],
    basis: &str,
) {
    f.registry(|doc| {
        let entry = doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        let operation = &mut entry["support_depth"]["enrollments"][0]["operations"][0];
        let mut rows = toml_edit::Array::new();
        for surface in surfaces {
            let mut identity = toml_edit::InlineTable::new();
            identity.insert("surface_kind", surface.surface_kind.as_str().into());
            identity.insert("command_path", surface.command_path.as_str().into());
            identity.insert("surface_id", surface.surface_id.as_str().into());
            let mut row = toml_edit::InlineTable::new();
            row.insert("surface", identity.into());
            row.insert(
                "templates",
                ["CORE"].into_iter().collect::<toml_edit::Array>().into(),
            );
            row.insert(
                "rationale",
                "CORE covers this whole `run --json` unit.".into(),
            );
            rows.push(row);
        }
        operation["classification_basis"] = toml_edit::value(basis);
        operation["classifications"] = toml_edit::value(rows);
    });
}
fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn record(root: &Path) -> DepthRecord {
    DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap()
}
fn advance(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) -> String {
    f.commit();
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    host.staging(&tip);
    set_local(f.root.path(), &tip);
    tip
}
fn ready(channel: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
    let (f, host) = delivered(true, channel);
    let root = f.root.path();
    let crate::depth_record::Event::Maintenance {
        request_recorded_at,
        ..
    } = record(root).event
    else {
        panic!("maintenance");
    };
    write(
        root,
        &marker(),
        stand_down::render_marker_toml("depth_fixture", "0.2.0", &request_recorded_at),
    );
    advance(&f, &host);
    (f, host)
}
fn marker() -> String {
    stand_down::marker_relative_path("docs/agents/lifecycle/depth_fixture-maintenance", "0.2.0")
}
fn prepare_args(root: &Path, write: bool) -> prepare_closeout::Args {
    prepare_closeout::Args {
        request: maintenance_request_path("depth_fixture").into(),
        commit: git(root, &["rev-parse", "HEAD"]),
        recorded_at: "2026-11-05T00:00:00Z".into(),
        write,
    }
}
fn preflight(commit: &str, status: &str, conclusion: &str) -> String {
    json!({"workflow_runs":[{"id":77,"run_attempt":1,"head_sha":commit,
        "path":".github/workflows/ci.yml","status":status,"conclusion":conclusion}]})
    .to_string()
}
fn prepare(root: &Path) -> Result<(), String> {
    let args = prepare_args(root, true);
    let commit = args.commit.clone();
    prepare_closeout::run_in_workspace_with_fetcher(root, args, &mut Vec::new(), |url| {
        assert_eq!(url, format!("https://api.github.com/repos/fixture/isolated/actions/runs?head_sha={commit}&per_page=100&page=1"));
        Ok(preflight(&commit, "completed", "success"))
    }).map_err(|e| e.to_string())
}

#[test]
fn view_eval_post_ma5_record_is_verified_in_scratch() {
    publication_proof(true);
}
#[test]
fn view_eval_authenticated_population_renders_verified_without_channel() {
    publication_proof(false);
}
fn publication_proof(channel: bool) {
    let (f, host) = ready(channel);
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    prepare(root).unwrap();
    let post = fs::read(root.join(RECORD)).unwrap();
    // Preserve the genuinely admitted post-MA5 record, but propose its entry in scratch only.
    let mut before: DepthRecord = serde_json::from_slice(&post).unwrap();
    assert!(matches!(
        before.acceptance_entries.as_slice(),
        [AcceptanceEntry::MaintenanceCloseout { .. }]
    ));
    before.acceptance_entries.clear();
    write(root, RECORD, serde_json::to_vec_pretty(&before).unwrap());
    advance(&f, &host);
    let context = f.resolve().unwrap();
    let evidence_host = context.evidence_host(root).unwrap();
    let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    write(scratch.path(), RECORD, post);
    let _guard = captured.bind_view(&view).unwrap();
    let evaluator =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &evidence_host, RECORD)
            .unwrap();
    let caller_before = snapshot(root);
    let scratch_before = snapshot_files(scratch.path());
    // Poison all route self-resolution after obtaining the caller's authenticated objects.
    let _no_resolution = fixture::install(Box::new(|| {
        panic!("view evaluation resolved its own authority")
    }));
    let (record, verdicts) = evaluator.evaluate().unwrap();
    assert_eq!(snapshot(root), caller_before);
    assert_eq!(snapshot_files(scratch.path()), scratch_before);
    for obligation in Profile::Synthetic.obligations(&record) {
        assert_eq!(verdicts.result(&record, &obligation), DepthResult::Verified);
        assert_eq!(
            verdicts.result(&before, &obligation),
            DepthResult::Unverified
        );
    }
    // Exercise the existing pure renderer explicitly; production generation still requests
    // production_results. No publication consumer is switched by this evaluator-only slice.
    use crate::support_matrix::{self, *};
    let target = &record.policy.selection.enrollment.targets[0];
    let identities = crate::depth_record::bound_evidence(scratch.path(), [&record]).unwrap();
    let facts = depth::derive_evaluated_depth_facts(
        Some(&record),
        true,
        target,
        &|p| identities.get(p).cloned(),
        &verdicts,
    )
    .unwrap();
    let row = SupportRow {
        agent: "depth_fixture".into(),
        version: "0.2.0".into(),
        target: target.clone(),
        manifest_support: ManifestSupportState::Unsupported,
        backend_support: BackendSupportState::Partial,
        uaa_support: UaaSupportState::Partial,
        pointer_promotion: PointerPromotionState::None,
        evidence_notes: vec![],
        depth: facts,
    };
    assert!(row
        .depth
        .as_ref()
        .unwrap()
        .promises
        .iter()
        .all(|p| p.qualified
            && p.obligations
                .iter()
                .all(|o| o.result == DepthResult::Verified)));
    let rendered = support_matrix::render_publication_bundle(&[row], String::new()).unwrap();
    let artifact: SupportMatrixArtifact = serde_json::from_str(&rendered.json).unwrap();
    assert!(artifact.rows[0]
        .depth
        .as_ref()
        .unwrap()
        .promises
        .iter()
        .all(|p| p.qualified));
    assert!(
        rendered.markdown.contains("`R1` verified"),
        "{}",
        rendered.markdown
    );
    // Rendering in scratch is the test's own write, after evaluation proved it wrote nothing.
    write(
        scratch.path(),
        support_matrix::JSON_OUTPUT_PATH,
        &rendered.json,
    );
    write(
        scratch.path(),
        support_matrix::MARKDOWN_OUTPUT_PATH,
        &rendered.markdown,
    );

    let reference = &record.execution.as_ref().unwrap().results[0].evidence[0];
    for path in [reference.path.as_str(), "crates/depth_fixture/src/lib.rs"] {
        let original = fs::read(scratch.path().join(path)).unwrap();
        write(scratch.path(), path, b"substituted evidence/dependency");
        // Tampering before binding cannot become a new trusted proposed input, even when
        // its altered bytes were included in the frozen source and E says verified.
        let evaluator = crate::depth_evidence::EvidenceVerdicts::bind_view(
            &view,
            &context,
            &evidence_host,
            RECORD,
        )
        .unwrap();
        let err = evaluator.evaluate().err().unwrap();
        assert!(err.contains("working bytes disagree"), "{err}");
        write(scratch.path(), path, original);
    }
    // A record-only seal also refuses an entry appended after this evaluation.
    let mut appended = record.clone();
    appended
        .acceptance_entries
        .push(AcceptanceEntry::Promotion {
            targets: vec![target.clone()],
            record_sha256: record.stated_sha256().unwrap(),
        });
    for obligation in Profile::Synthetic.obligations(&record) {
        assert_eq!(
            verdicts.result(&appended, &obligation),
            DepthResult::Unverified
        );
    }
    // The caller stays current through all three pre-evaluation rechecks, then its host
    // identity changes at the first post-evaluation recheck. No sealed result may escape.
    let evaluator =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &evidence_host, RECORD)
            .unwrap();
    let endpoint = "repos/fixture/isolated";
    let identity = f.api.0.lock().unwrap()[endpoint].clone();
    f.api.1.lock().unwrap().insert(
        endpoint.into(),
        [
            identity.clone(),
            identity.clone(),
            identity,
            json!({"id":999}),
        ]
        .into_iter()
        .collect(),
    );
    let err = evaluator.evaluate().err().unwrap();
    assert!(
        err.contains("repository id, owner id or full name differs"),
        "{err}"
    );
    assert!(f.api.1.lock().unwrap()[endpoint].is_empty());
    assert!(!scratch.path().join(".git").exists());
    assert_eq!(snapshot(root), caller_before);
}

fn snapshot_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let source = Source::workspace(root).unwrap();
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        if name == ".git" {
            continue;
        }
        if entry.file_type().unwrap().is_dir() {
            paths.extend(source.paths(&name).unwrap());
        } else {
            paths.push(name);
        }
    }
    paths
        .into_iter()
        .filter(|p| root.join(p).is_file())
        .map(|p| {
            let b = source.optional_bytes(&p).unwrap().unwrap();
            (p, b)
        })
        .collect()
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    status: String,
    files: BTreeMap<String, Vec<u8>>,
    git: BTreeMap<PathBuf, Vec<u8>>,
}
fn snapshot(root: &Path) -> Snapshot {
    let mut git_files = BTreeMap::new();
    let mut pending = vec![root.join(".git")];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else {
                git_files.insert(
                    entry.path().strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    Snapshot {
        status: git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
        files: snapshot_files(root),
        git: git_files,
    }
}

fn simple() -> Fixture {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    write(f.root.path(), RECORD, f.retained());
    f.commit();
    f
}
#[test]
fn view_eval_refuses_scratch_git_before_and_after_binding() {
    for after_binding in [false, true] {
        for kind in 0..3 {
            let f = simple();
            let root = f.root.path();
            let context = f.resolve().unwrap();
            let host = context.evidence_host(root).unwrap();
            let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
            let proposal = captured.proposal(root).unwrap();
            let scratch = tempfile::tempdir().unwrap();
            let view = proposal.isolated_view(root, scratch.path()).unwrap();
            let _guard = captured.bind_view(&view).unwrap();
            let evaluator = after_binding.then(|| {
                crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD)
                    .unwrap()
            });
            match kind {
                0 => fs::create_dir(scratch.path().join(".git")).unwrap(),
                1 => write(scratch.path(), ".git", "gitdir: /foreign\n"),
                _ => std::os::unix::fs::symlink("missing", scratch.path().join(".git")).unwrap(),
            }
            let err = if let Some(evaluator) = evaluator {
                evaluator.evaluate().err().unwrap()
            } else {
                crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD)
                    .err()
                    .unwrap()
            };
            assert!(
                err.contains("scratch cannot acquire Git authority"),
                "{err}"
            );
        }
    }
}
#[test]
fn view_eval_refuses_moved_caller_before_and_after_binding() {
    for after_binding in [false, true] {
        let f = simple();
        let root = f.root.path();
        let context = f.resolve().unwrap();
        let host = context.evidence_host(root).unwrap();
        let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
        let proposal = captured.proposal(root).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let view = proposal.isolated_view(root, scratch.path()).unwrap();
        let _guard = captured.bind_view(&view).unwrap();
        let evaluator = after_binding.then(|| {
            crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD)
                .unwrap()
        });
        git(root, &["commit", "--allow-empty", "-m", "moved caller"]);
        let err = if let Some(evaluator) = evaluator {
            evaluator.evaluate().err().unwrap()
        } else {
            crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD)
                .err()
                .unwrap()
        };
        assert!(err.contains("caller workspace"), "{err}");
    }
}
#[test]
fn view_eval_refuses_changed_proposal_after_binding() {
    for path in [RECORD, "docs/input.txt", "docs/new-proposal.txt"] {
        let f = simple();
        let root = f.root.path();
        let context = f.resolve().unwrap();
        let host = context.evidence_host(root).unwrap();
        let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
        let proposal = captured.proposal(root).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let view = proposal.isolated_view(root, scratch.path()).unwrap();
        let _guard = captured.bind_view(&view).unwrap();
        let evaluator =
            crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD)
                .unwrap();
        write(scratch.path(), path, "changed after binding");
        let err = evaluator.evaluate().err().unwrap();
        assert!(
            err.contains("proposal changed after evaluation binding"),
            "{err}"
        );
        assert_eq!(
            fs::read_to_string(scratch.path().join(path)).unwrap(),
            "changed after binding"
        );
    }
}
#[test]
fn view_eval_never_resolves_scratch_authority_and_requires_the_callers_guard() {
    let f = simple();
    let root = f.root.path();
    let _installed = f.install();
    let context = f.resolve().unwrap();
    let host = context.evidence_host(root).unwrap();
    let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    assert!(
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD).is_err()
    );
    let guard = captured.bind_view(&view).unwrap();
    assert!(route(scratch.path()).is_err());
    assert!(evidence_host(scratch.path()).is_err());
    assert!(context.evidence_host(scratch.path()).is_err());
    let _no_resolution = fixture::install(Box::new(|| panic!("self-resolution attempted")));
    let evaluator =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD).unwrap();
    // A registry/Git binding alone cannot authenticate a population. No fake reader is installed.
    assert!(evaluator.evaluate().is_err());
    let evaluator =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD).unwrap();
    drop(guard);
    assert!(evaluator.evaluate().is_err());
}
#[test]
fn view_eval_refuses_foreign_context_and_borrowed_host() {
    let f = simple();
    let other = simple();
    write(other.root.path(), "docs/input.txt", "distinct caller");
    other.commit();
    let root = f.root.path();
    let context = f.resolve().unwrap();
    let host = context.evidence_host(root).unwrap();
    let foreign = other.resolve().unwrap();
    let foreign_host = foreign.evidence_host(other.root.path()).unwrap();
    let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    let err =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &foreign, &foreign_host, RECORD)
            .err()
            .unwrap();
    assert!(err.contains("another caller"), "{err}");
    let err =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &foreign_host, RECORD)
            .err()
            .unwrap();
    assert!(err.contains("host differs"), "{err}");
    assert!(crate::depth_evidence::EvidenceVerdicts::bind_view(
        &view,
        &context,
        &host,
        "docs/forged-record.json"
    )
    .is_err());
}

#[test]
fn view_eval_provenance_refuses_missing_objects_and_scratch_objects_without_fetching() {
    let f = simple();
    let root = f.root.path();
    let context = f.resolve().unwrap();
    let host = context.evidence_host(root).unwrap();
    let captured = Route(f.resolve().unwrap()).capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    let _guard = captured.bind_view(&view).unwrap();
    let evaluator =
        crate::depth_evidence::EvidenceVerdicts::bind_view(&view, &context, &host, RECORD).unwrap();
    let before = snapshot(root);
    let commit = git(root, &["rev-parse", "HEAD"]);
    evaluator.probe_provenance(root, &commit).unwrap();
    let err = evaluator
        .probe_provenance(scratch.path(), &commit)
        .unwrap_err();
    assert!(err.contains("caller's Git object store"), "{err}");
    assert!(evaluator.probe_provenance(root, &"f".repeat(40)).is_err());
    assert_eq!(snapshot(root), before);
    assert!(!root.join(".git/FETCH_HEAD").exists());
    assert!(!scratch.path().join(".git").exists());
}
