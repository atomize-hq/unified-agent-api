//! Both public MA5 writers on the connected synthetic MA1 → MA2 → MA4 evidence chain.
#![cfg(target_os = "linux")]
use super::*;
use super::{
    ma4_route::run_as,
    prepare::{acquire, workspace},
};
use crate::{
    agent_lifecycle::maintenance_request_path,
    agent_maintenance::{
        closeout, execute,
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

fn delivered(classified: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
    let (f, host) = frozen(classified);
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let binary = executor(bin.path());
    {
        let _installed = f.install();
        let _reader = host.install();
        let _gates = execute::gate_stub::stub();
        run_as(root, &binary, "authority").unwrap();
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
fn ready() -> (Fixture, crate::depth_evidence::SyntheticHost) {
    let (f, host) = delivered(true);
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
fn close(root: &Path) -> Result<(), String> {
    closeout::run_in_workspace(
        root,
        closeout::Args {
            request: maintenance_request_path("depth_fixture").into(),
            closeout: CLOSEOUT.into(),
        },
        &mut Vec::new(),
    )
    .map_err(|e| e.to_string())
}
/// A maintainer-authored closeout, committed before the writer captures its clean route.
fn author(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) {
    let root = f.root.path();
    let findings = closeout::derive_findings(root, "depth_fixture", "0.2.0").unwrap();
    let closeout = closeout::MaintenanceCloseout {
        request_ref: maintenance_request_path("depth_fixture"),
        request_sha256: digest(
            &fs::read(root.join(maintenance_request_path("depth_fixture"))).unwrap(),
        ),
        resolved_findings: findings.resolved_findings,
        deferred_findings: findings.deferred_findings,
        wrapper_only_baseline_ref: Some(
            "cli_manifests/depth_fixture/reports/0.2.0/coverage.any.json".into(),
        ),
        wrapper_only_dispositions: vec![],
        preflight_passed: true,
        recorded_at: "2026-11-05T00:00:00Z".into(),
        commit: git(root, &["rev-parse", "HEAD"]),
    };
    write(
        root,
        CLOSEOUT,
        closeout::serialize_closeout_json(&closeout).unwrap(),
    );
    advance(f, host);
}
fn audit(root: &Path) {
    use crate::agent_maintenance::audit_status;
    assert_eq!(
        audit_status::run_in_workspace(
            root,
            audit_status::Args {
                request: maintenance_request_path("depth_fixture").into(),
                expect_target_version: Some("0.2.0".into()),
                emit_json: None,
                workspace_root: None,
            },
            &mut Vec::new()
        )
        .unwrap(),
        audit_status::AuditStatusOutcome::Clean
    );
    let record = record(root);
    let verdicts = crate::depth_evidence::evaluate_isolated_record(root, &record).unwrap();
    assert_eq!(
        Profile::Synthetic
            .obligations(&record)
            .iter()
            .map(|o| verdicts.result(&record, o))
            .collect::<Vec<_>>(),
        vec![DepthResult::Verified; 11]
    );
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    head: String,
    status: String,
    index: Vec<u8>,
    files: Vec<(String, Vec<u8>)>,
}
fn snapshot(root: &Path) -> Snapshot {
    let files = git(root, &["ls-files"]);
    Snapshot {
        head: git(root, &["rev-parse", "HEAD"]),
        status: git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
        index: fs::read(root.join(".git/index")).unwrap(),
        files: files
            .lines()
            .map(|p| (p.into(), fs::read(root.join(p)).unwrap()))
            .collect(),
    }
}
fn assert_entry(root: &Path, before: &DepthRecord) {
    let after = record(root);
    assert_eq!(
        (
            &after.event,
            &after.policy,
            &after.obligations,
            &after.execution
        ),
        (
            &before.event,
            &before.policy,
            &before.obligations,
            &before.execution
        )
    );
    assert_eq!(
        after.acceptance_entries,
        [AcceptanceEntry::MaintenanceCloseout {
            closeout: crate::depth_record::FileRef {
                path: CLOSEOUT.into(),
                sha256: digest(&fs::read(root.join(CLOSEOUT)).unwrap())
            },
            record_sha256: before.stated_sha256().unwrap(),
        }]
    );
}

fn assert_owned_outputs(root: &Path) {
    let changed = git(root, &["diff", "HEAD", "--name-only"]);
    let added = git(root, &["ls-files", "--others", "--exclude-standard"]);
    for path in changed.lines().chain(added.lines()) {
        assert!(
            [
                RECORD,
                CLOSEOUT,
                "docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md",
                "docs/agents/lifecycle/depth_fixture-maintenance/governance/remediation-log.md",
                "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json",
            ]
            .contains(&path),
            "unowned MA5 output: {path}"
        );
    }
}

#[test]
fn ma5_closeout_writer_completes_connected_chain() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    audit(root);
    author(&f, &host);
    let before = record(root);
    close(root).unwrap();
    assert_entry(root, &before);
    assert_owned_outputs(root);
    assert!(fs::read_to_string(
        root.join("docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md")
    )
    .unwrap()
    .contains("generated-by: xtask close-agent-maintenance"));
    // Qualified publication is not exercised until B.7b.
}
#[test]
fn ma5_prepare_writer_completes_connected_chain_from_isolated_preflight() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    audit(root);
    let before = record(root);
    let reads = f.api.2.load(std::sync::atomic::Ordering::SeqCst);
    prepare(root).unwrap();
    assert!(f.api.2.load(std::sync::atomic::Ordering::SeqCst) > reads);
    assert_entry(root, &before);
    assert_owned_outputs(root);
}

#[test]
fn ma5_writers_keep_production_depth_refusal_without_a_route() {
    let (f, host) = ready();
    let root = f.root.path();
    {
        let _installed = f.install();
        let _reader = host.install();
        author(&f, &host);
    }
    let before = snapshot(root);
    let err = close(root).unwrap_err();
    assert!(err.contains("support_depth"), "{err}");
    let args = prepare_args(root, true);
    let commit = args.commit.clone();
    let err = prepare_closeout::run_in_workspace_with_fetcher(root, args, &mut Vec::new(), |url| {
        assert!(url.starts_with(
            "https://api.github.com/repos/atomize-hq/unified-agent-api/actions/runs?"
        ));
        Ok(preflight(&commit, "completed", "success"))
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("support_depth"), "{err}");
    // Even with a valid depth record and preflight the production owning host cannot admit.
    let err = crate::agent_maintenance::closeout_proposal::Proposal::capture(root)
        .unwrap()
        .plan(
            root,
            vec![(CLOSEOUT.into(), fs::read(root.join(CLOSEOUT)).unwrap())],
        )
        .unwrap_err();
    assert!(err.contains("not independently satisfied"), "{err}");
    assert_eq!(snapshot(root), before);
}

#[test]
fn ma5_writers_reauthenticate_unverified_population_despite_verified_e_labels() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    author(&f, &host);
    let before = snapshot(root);
    let crate::depth_record::Event::Maintenance {
        request_recorded_at,
        ..
    } = record(root).event
    else {
        panic!("maintenance");
    };
    let runs = format!(
        "actions/workflows/ci.yml/runs?created=%3E%3D{}&per_page=100&page=1",
        &request_recorded_at[..10]
    );
    let edits: [fn(&crate::depth_evidence::SyntheticHost, &str); 3] = [
        |h, runs| h.add_attempt(runs, |job| job["status"] = json!("in_progress")),
        |h, runs| {
            h.add_attempt(runs, |job| {
                job["steps"][0]["conclusion"] = json!("failure");
                job["conclusion"] = json!("failure");
            })
        },
        |h, _| h.revoke_adoption(),
    ];
    assert!(record(root)
        .execution
        .unwrap()
        .results
        .iter()
        .all(|r| r.result == DepthResult::Verified));
    for edit in edits {
        let changed = host.fork();
        edit(&changed, &runs);
        let _changed = changed.install();
        for writer in [close, prepare] {
            let err = writer(root).unwrap_err();
            assert!(err.contains("not independently satisfied"), "{err}");
            assert_eq!(snapshot(root), before);
        }
    }
}

#[test]
fn ma5_writers_refuse_wrong_generation_policy_or_obligations_without_writes() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    author(&f, &host);
    let original = fs::read(root.join(RECORD)).unwrap();
    for edit in 0..3 {
        let mut changed = record(root);
        match edit {
            0 => {
                let crate::depth_record::Event::Maintenance {
                    request_recorded_at,
                    ..
                } = &mut changed.event
                else {
                    panic!("maintenance");
                };
                *request_recorded_at = "2026-11-01T00:00:00Z".into();
            }
            1 => changed.policy.rules_revision = "other-governing-rules".into(),
            _ => changed.obligations.as_mut().unwrap().acquired_inputs[0].sha256 = "0".repeat(64),
        }
        // Well-formed identities do not make the altered parts current or authenticated.
        changed.policy.identity = crate::depth_record::part_identity(&changed.policy).unwrap();
        let o = changed.obligations.as_mut().unwrap();
        o.policy_identity = changed.policy.identity.clone();
        o.identity = crate::depth_record::part_identity(o).unwrap();
        let e = changed.execution.as_mut().unwrap();
        e.policy_identity = changed.policy.identity.clone();
        e.obligations_identity = o.identity.clone();
        e.identity = crate::depth_record::part_identity(e).unwrap();
        let bytes = serde_json::to_vec_pretty(&changed).unwrap();
        DepthRecord::parse(
            std::str::from_utf8(&bytes).unwrap(),
            "depth_fixture",
            "0.2.0",
        )
        .unwrap();
        write(root, RECORD, bytes);
        advance(&f, &host);
        let before = snapshot(root);
        for writer in [close, prepare] {
            assert!(writer(root).is_err(), "accepted changed part {edit}");
            assert_eq!(snapshot(root), before);
        }
        write(root, RECORD, &original);
        advance(&f, &host);
    }
}

#[test]
fn ma5_writers_refuse_missing_or_wrong_generation_marker() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    author(&f, &host);
    for present in [false, true] {
        if present {
            write(
                root,
                &marker(),
                stand_down::render_marker_toml("depth_fixture", "0.2.0", "2026-11-01T00:00:00Z"),
            );
        } else {
            fs::remove_file(root.join(marker())).unwrap();
        }
        advance(&f, &host);
        let before = snapshot(root);
        for writer in [close, prepare] {
            assert!(writer(root).is_err());
            assert_eq!(snapshot(root), before);
        }
    }
}

#[test]
fn ma5_closeout_plan_and_prepare_fetch_refuse_captured_route_drift() {
    let (f, host) = ready();
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
    author(&f, &host);
    let linked = closeout::load_linked_closeout(
        root,
        Path::new(&maintenance_request_path("depth_fixture")),
        Path::new(CLOSEOUT),
    )
    .unwrap();
    let before = snapshot(root);
    let plan = closeout::plan_closeout_mutations(root, &linked).unwrap();
    let endpoint = "repos/fixture/isolated";
    let changed = json!({"id":999,"owner":{"id":20},"full_name":"fixture/isolated"});
    f.api.put(endpoint, changed.clone());
    assert!(plan.apply(root).is_err());
    assert_eq!(snapshot(root), before);
    f.host();
    let args = prepare_args(root, true);
    let commit = args.commit.clone();
    let err = prepare_closeout::run_in_workspace_with_fetcher(root, args, &mut Vec::new(), |_| {
        f.api.put(endpoint, changed.clone());
        Ok(preflight(&commit, "completed", "success"))
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("isolated"), "{err}");
    assert_eq!(snapshot(root), before);
}

#[test]
fn ma5_preflight_repository_is_bound_to_the_captured_context() {
    use crate::agent_maintenance::closeout_proposal::Proposal;
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let production = Proposal::capture(root).unwrap();
    assert_eq!(production.preflight_repository(root).unwrap(), None);
    let _installed = f.install();
    let proposal = route(root).unwrap().unwrap().proposal(root).unwrap();
    assert_eq!(
        proposal.preflight_repository(root).unwrap(),
        Some("fixture/isolated")
    );
    assert_eq!(production.preflight_repository(root).unwrap(), None);
    let foreign = tempfile::tempdir().unwrap();
    assert!(proposal.preflight_repository(foreign.path()).is_err());
    let before = snapshot(root);
    f.api.put(
        "repos/fixture/isolated",
        json!({"id":999,"owner":{"id":20},"full_name":"fixture/isolated"}),
    );
    assert!(proposal.preflight_repository(root).is_err());
    let err = prepare_closeout::run_in_workspace_with_fetcher(
        root,
        prepare_args(root, true),
        &mut Vec::new(),
        |_| panic!("a failed installed route must refuse before preflight"),
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("isolated"), "{err}");
    assert_eq!(snapshot(root), before);
}

/// This CP-P4 check needs no socket: the repository comes from the captured proposal's context.
#[test]
fn ma5_preflight_uses_captured_host_and_preserves_selection_and_terminality() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let _installed = f.install();
    let proposal = route(root).unwrap().unwrap().proposal(root).unwrap();
    let repository = proposal.preflight_repository(root).unwrap().unwrap();
    let commit = git(root, &["rev-parse", "HEAD"]);
    let resolve = |runs: Value| {
        closeout::ResolvedPreflight::resolve_in_repository(&commit, repository, |url| {
            assert_eq!(url, format!("https://api.github.com/repos/fixture/isolated/actions/runs?head_sha={commit}&per_page=100&page=1"));
            Ok(runs.to_string())
        })
    };
    let good: Value = serde_json::from_str(&preflight(&commit, "completed", "success")).unwrap();
    assert!(resolve(good.clone()).unwrap().passed);
    let mut other = good.clone();
    other["workflow_runs"][0]["head_sha"] = json!("0".repeat(40));
    assert!(resolve(other)
        .unwrap_err()
        .to_string()
        .contains("fixture/isolated"));
    let mut other = good.clone();
    other["workflow_runs"][0]["path"] = json!(".github/workflows/other.yml");
    assert!(resolve(other).is_err());
    // A successful producer inside an incomplete workflow is not terminal whole-workflow CI.
    let mut pending = good.clone();
    pending["workflow_runs"][0]["status"] = json!("in_progress");
    assert!(resolve(pending)
        .unwrap_err()
        .to_string()
        .contains("not final"));
    let mut disagree = good.clone();
    let mut failed = good["workflow_runs"][0].clone();
    failed["id"] = json!(78);
    failed["conclusion"] = json!("failure");
    disagree["workflow_runs"]
        .as_array_mut()
        .unwrap()
        .push(failed.clone());
    assert!(resolve(disagree)
        .unwrap_err()
        .to_string()
        .contains("disagrees"));
    let mut rerun = good;
    failed["id"] = json!(77);
    failed["run_attempt"] = json!(2);
    rerun["workflow_runs"].as_array_mut().unwrap().push(failed);
    assert!(!resolve(rerun).unwrap().passed);
}

/// The full chain above must use MA4's channel. This separate fixture stops before MA4 so the
/// routed readers, preview and missing-E refusal can also be checked inside a socket sandbox.
#[test]
fn ma5_readers_and_preview_are_routed_but_missing_e_never_writes() {
    let (f, host) = frozen(true);
    let root = f.root.path();
    let _installed = f.install();
    let _reader = host.install();
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
    let args = prepare_args(root, false);
    let commit = args.commit.clone();
    let before = snapshot(root);
    let mut preview = Vec::new();
    prepare_closeout::run_in_workspace_with_fetcher(root, args, &mut preview, |url| {
        assert!(url.starts_with("https://api.github.com/repos/fixture/isolated/"));
        Ok(preflight(&commit, "completed", "success"))
    })
    .unwrap();
    assert!(String::from_utf8(preview)
        .unwrap()
        .contains("Nothing written."));
    assert_eq!(snapshot(root), before);
    assert!(!root.join(CLOSEOUT).exists());
    let err = prepare(root).unwrap_err();
    assert!(err.contains("requires current E"), "{err}");
    assert_eq!(snapshot(root), before);
    author(&f, &host);
    let before = snapshot(root);
    let err = close(root).unwrap_err();
    assert!(err.contains("requires current E"), "{err}");
    assert_eq!(snapshot(root), before);
}
