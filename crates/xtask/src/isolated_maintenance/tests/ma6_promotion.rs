//! MA6 owns promotion through one captured caller route; scratch confers no authority.
use super::*;
use crate::{
    agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan},
    manifest_promotion::{self, Args},
};
const TARGET: &str = "x86_64-unknown-linux-gnu";
fn set_local(root: &Path, tip: &str) {
    git(root, &["update-ref", "refs/remotes/origin/staging", tip]);
}
fn args(root: &Path) -> Args {
    Args {
        agent: "depth_fixture".into(),
        version: "0.2.0".into(),
        passed_targets: vec![TARGET.into()],
        validation_commit: git(root, &["rev-parse", "HEAD"]),
        write: false,
    }
}
fn pending() -> Fixture {
    let (f, a) = super::prepare::workspace();
    let root = f.root.path();
    {
        let _installed = f.install();
        apply_prepare_plan(root, &build_prepare_plan(root, &a).unwrap()).unwrap();
        f.commit();
        super::prepare::acquire(root, "0.2.0");
        f.commit();
        apply_prepare_plan(root, &build_prepare_plan(root, &a).unwrap()).unwrap();
        f.commit();
    }
    set_local(root, &args(root).validation_commit);
    f
}
#[test]
fn ma6_routed_scratch_reaches_owning_predicate_without_closeout() {
    let f = pending();
    let resolutions = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (count, baseline, api) = (resolutions.clone(), f.baseline.clone(), f.api.clone());
    let _installed = fixture::install(Box::new(move || {
        count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        (
            fixture::profile_with(baseline.clone(), &scope::SYNTHETIC),
            api.clone(),
        )
    }));
    let before = snapshot(f.root.path());
    let err = manifest_promotion::build_plan(f.root.path(), &args(f.root.path()))
        .err()
        .unwrap();
    assert!(
        err.contains("current retained maintenance closeout entry"),
        "{err}"
    );
    assert_eq!(resolutions.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(snapshot(f.root.path()), before);
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    head: String,
    status: String,
    integration_tip: String,
    index: Vec<u8>,
    files: Vec<(String, Option<Vec<u8>>)>,
}
fn snapshot(root: &Path) -> Snapshot {
    Snapshot {
        head: git(root, &["rev-parse", "HEAD"]),
        status: git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
        integration_tip: git(
            root,
            &[
                "for-each-ref",
                "--format=%(objectname)",
                "refs/remotes/origin/staging",
            ],
        ),
        index: fs::read(root.join(".git/index")).unwrap(),
        files: git(
            root,
            &["ls-files", "--cached", "--others", "--exclude-standard"],
        )
        .lines()
        .map(|p| (p.into(), fs::read(root.join(p)).ok()))
        .collect(),
    }
}
#[test]
fn ma6_no_route_keeps_production_depth_refusal() {
    let f = pending();
    let root = f.root.path();
    let before = snapshot(root);
    let err = manifest_promotion::build_plan(root, &args(root))
        .err()
        .unwrap();
    assert!(err.contains("support_depth"), "{err}");
    assert_eq!(snapshot(root), before);
}
#[test]
fn ma6_refuses_validation_commit_away_from_authenticated_tip() {
    let f = pending();
    let root = f.root.path();
    let tip = args(root).validation_commit;
    {
        let _installed = f.install();
        set_local(root, &f.baseline);
        let before = snapshot(root);
        let err = manifest_promotion::build_plan(root, &args(root))
            .err()
            .unwrap();
        assert!(err.contains("HEAD/integration tip"), "{err}");
        assert_eq!(snapshot(root), before);
        set_local(root, &tip);
    }
    git(
        root,
        &["commit", "--allow-empty", "-m", "unpushed candidate"],
    );
    let head = args(root).validation_commit;
    for base in [&tip, &f.baseline] {
        f.api.put(
            &format!("repos/fixture/isolated/compare/{base}...{head}"),
            json!({"status":"ahead","merge_base_commit":{"sha":base}}),
        );
    }
    f.api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    // Both local refs agree on H, but the host still authenticates the older T.
    set_local(root, &head);
    let _installed = f.install();
    let before = snapshot(root);
    let err = manifest_promotion::build_plan(root, &args(root))
        .err()
        .unwrap();
    assert!(err.contains("authenticated protected tip"), "{err}");
    assert_eq!(snapshot(root), before);
}
#[test]
fn ma6_unbound_and_forged_scratch_cannot_supply_a_registry_or_authority() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    let _installed = f.install();
    let captured = route(root).unwrap().unwrap().capture(root).unwrap();
    let proposal = captured.proposal(root).unwrap();
    let before = snapshot(root);
    let scratch = tempfile::tempdir().unwrap();
    proposal.materialize(scratch.path()).unwrap();
    assert!(registry(scratch.path()).is_err());
    assert!(proposal.isolated_view(root, scratch.path()).is_err());
    let scratch = tempfile::tempdir().unwrap();
    let view = proposal.isolated_view(root, scratch.path()).unwrap();
    let original = fs::read(scratch.path().join(REGISTRY_RELATIVE_PATH)).unwrap();
    write(scratch.path(), REGISTRY_RELATIVE_PATH, b"forged registry");
    assert!(captured.bind_view(&view).is_err());
    write(scratch.path(), REGISTRY_RELATIVE_PATH, original);
    let guard = captured.bind_view(&view).unwrap();
    assert_eq!(registry(scratch.path()).unwrap(), *captured.registry());
    assert!(route(scratch.path()).is_err());
    assert!(evidence_host(scratch.path()).is_err());
    drop(guard);
    assert!(registry(scratch.path()).is_err());
    assert_eq!(snapshot(root), before);
}

// Full chains use MA4's Linux socket channel; the lead runs these outside the socket sandbox.
#[cfg(target_os = "linux")]
mod connected {
    use super::super::{
        ma4_route::run_as,
        prepare::{acquire, workspace},
    };
    use super::*;
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
        let root = f.root.path();
        f.commit();
        let tip = git(root, &["rev-parse", "HEAD"]);
        host.staging(&tip);
        set_local(root, &tip);
        // Every generation/code edit is committed and published as the authenticated tip.
        assert_eq!(git(root, &["status", "--porcelain"]), "");
        assert_eq!(git(root, &["rev-parse", "origin/staging"]), tip);
        assert_eq!(f.resolve().unwrap().protected_tip(root).unwrap(), tip);
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

    fn candidate(f: &Fixture, tip: &str) -> String {
        git(f.root.path(), &["add", "."]);
        git(f.root.path(), &["commit", "-m", "MA6 chain candidate"]);
        let head = git(f.root.path(), &["rev-parse", "HEAD"]);
        for base in [tip, &f.baseline] {
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
    fn admit_and_observe(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) -> String {
        let root = f.root.path();
        let parent = git(root, &["rev-parse", "HEAD"]);
        let head = candidate(f, &parent);
        crate::depth_acceptance::check_integration(root, &parent).unwrap();
        f.host();
        host.staging(&head);
        set_local(root, &head);
        crate::depth_acceptance::check_integration(root, &parent).unwrap();
        head
    }
    fn admitted_ma5(prepare_writer: bool) -> (Fixture, crate::depth_evidence::SyntheticHost) {
        let (f, host) = ready();
        {
            let _installed = f.install();
            let _reader = host.install();
            audit(f.root.path());
            if prepare_writer {
                prepare(f.root.path()).unwrap();
            } else {
                author(&f, &host);
                close(f.root.path()).unwrap();
            }
            admit_and_observe(&f, &host);
            assert!(matches!(
                record(f.root.path()).acceptance_entries.as_slice(),
                [AcceptanceEntry::MaintenanceCloseout { .. }]
            ));
        }
        (f, host)
    }
    fn promote(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) {
        let root = f.root.path();
        let retained = record(root);
        assert!(matches!(
            retained.acceptance_entries.as_slice(),
            [AcceptanceEntry::MaintenanceCloseout { .. }]
        ));
        assert_eq!(
            fs::read_to_string(root.join("cli_manifests/depth_fixture/latest_validated.txt"))
                .unwrap(),
            "0.1.0\n"
        );
        let before = snapshot(root);
        let plan = manifest_promotion::build_plan(root, &args(root)).unwrap();
        assert_eq!(snapshot(root), before);
        for path in [
            "cli_manifests/depth_fixture/current.json",
            "cli_manifests/depth_fixture/versions/0.2.0.json",
            "cli_manifests/depth_fixture/pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
            "cli_manifests/depth_fixture/pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
            crate::support_matrix::JSON_OUTPUT_PATH,
            crate::support_matrix::MARKDOWN_OUTPUT_PATH,
            crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH,
            crate::capability_matrix::DEFAULT_OUT_PATH,
        ] {
            assert!(plan.paths().contains(&Path::new(path)), "missing {path}");
        }
        plan.apply(root).unwrap();
        let promoted = record(root);
        assert_eq!(
            promoted.stated_sha256().unwrap(),
            retained.stated_sha256().unwrap()
        );
        assert_eq!(
            promoted.acceptance_entries[0],
            retained.acceptance_entries[0]
        );
        assert_eq!(promoted.acceptance_entries.len(), 2);
        assert_eq!(
            promoted.acceptance_entries[1],
            AcceptanceEntry::Promotion {
                targets: vec![TARGET.into()],
                record_sha256: retained.stated_sha256().unwrap(),
            }
        );
        assert!(!root.join(marker()).exists());
        for path in [
            "latest_validated.txt",
            "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
            "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
        ] {
            assert_eq!(
                fs::read_to_string(root.join("cli_manifests/depth_fixture").join(path)).unwrap(),
                "0.2.0\n"
            );
        }
        assert_eq!(
            fs::read(root.join("cli_manifests/depth_fixture/current.json")).unwrap(),
            fs::read(root.join("cli_manifests/depth_fixture/snapshots/0.2.0/union.json")).unwrap()
        );
        let artifact: crate::support_matrix::SupportMatrixArtifact = serde_json::from_slice(
            &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
        )
        .unwrap();
        let row = artifact
            .rows
            .iter()
            .find(|r| r.agent == "depth_fixture" && r.version == "0.2.0")
            .unwrap();
        assert!(row
            .depth
            .as_ref()
            .unwrap()
            .promises
            .iter()
            .all(|p| !p.qualified
                && p.obligations
                    .iter()
                    .all(|o| o.result == DepthResult::Unverified)));
        admit_and_observe(f, host);
    }
    #[test]
    fn ma6_positive_promotion_after_each_ma5_writer_with_in1_admission_and_observation() {
        for prepare_writer in [false, true] {
            let (f, host) = admitted_ma5(prepare_writer);
            let _installed = f.install();
            let _reader = host.install();
            promote(&f, &host);
        }
    }
    fn later_generation(f: &Fixture, host: &crate::depth_evidence::SyntheticHost) {
        let root = f.root.path();
        let request = maintenance_request_path("depth_fixture");
        let mut next = crate::agent_maintenance::prepare::args_from_request_in_workspace(
            root,
            Path::new(&request),
            false,
            true,
        )
        .unwrap();
        next.target_version = "0.3.0".into();
        next.latest_stable = "0.4.0".into();
        next.branch_name = "automation/depth_fixture-maintenance-0.3.0".into();
        next.request_recorded_at = "2026-11-06T00:00:00Z".into();
        next.request_commit = git(root, &["rev-parse", "HEAD"]);
        apply_prepare_plan(root, &build_prepare_plan(root, &next).unwrap()).unwrap();
        advance(f, host);
        acquire(root, "0.3.0");
        advance(f, host);
        apply_prepare_plan(root, &build_prepare_plan(root, &next).unwrap()).unwrap();
        write(
            root,
            &stand_down::marker_relative_path(
                "docs/agents/lifecycle/depth_fixture-maintenance",
                "0.3.0",
            ),
            stand_down::render_marker_toml("depth_fixture", "0.3.0", &next.request_recorded_at),
        );
        advance(f, host);
        prepare(root).unwrap();
        advance(f, host);
        let replaced = fs::read_to_string(root.join(&request)).unwrap();
        assert!(replaced.contains("0.3.0"));
        let closed: Value =
            serde_json::from_slice(&fs::read(root.join(CLOSEOUT)).unwrap()).unwrap();
        assert_eq!(
            closed["request_sha256"],
            digest(&fs::read(root.join(request)).unwrap())
        );
        assert!(!root
            .join("cli_manifests/depth_fixture/reports/0.3.0/depth-record.json")
            .exists());
    }
    #[test]
    fn ma6_ordinary_replay_generation_replaces_working_files_without_a_depth_record() {
        let (f, host) = frozen(true);
        let retained = record(f.root.path());
        let _installed = f.install();
        let _reader = host.install();
        later_generation(&f, &host);
        assert_eq!(record(f.root.path()), retained);
    }
    #[test]
    fn ma6_a5_replay_first_promotion_uses_retained_closeout_after_ordinary_later_generation() {
        let (f, host) = admitted_ma5(true);
        let root = f.root.path();
        let checkpoint = git(root, &["rev-parse", "HEAD"]);
        let producer_tree = git(root, &["rev-parse", "HEAD:crates"]);
        let retained = record(root);
        let old_closeout = fs::read(root.join(CLOSEOUT)).unwrap();
        let _installed = f.install();
        let _reader = host.install();
        git(root, &["checkout", "-b", "ordinary-replay", &checkpoint]);
        later_generation(&f, &host);
        assert_eq!(git(root, &["rev-parse", "HEAD:crates"]), producer_tree);
        assert_eq!(record(root), retained);
        assert_ne!(fs::read(root.join(CLOSEOUT)).unwrap(), old_closeout);
        promote(&f, &host);
        // A sibling forks from the same admitted MA5 checkpoint, never from a promoted state.
        git(root, &["checkout", "-b", "invalidated-replay", &checkpoint]);
        f.host();
        host.staging(&checkpoint);
        set_local(root, &checkpoint);
        later_generation(&f, &host);
        let dependency = "crates/depth_fixture/src/lib.rs";
        let source = fs::read_to_string(root.join(dependency)).unwrap();
        let changed = source.replace(
            "MAX_PROMPT_BYTES: usize = 8 * 1024",
            "MAX_PROMPT_BYTES: usize = 1",
        );
        assert_ne!(source, changed);
        write(root, dependency, changed);
        advance(&f, &host);
        assert_ne!(git(root, &["rev-parse", "HEAD:crates"]), producer_tree);
        assert_eq!(record(root), retained);
        let before = snapshot(root);
        let refusal = manifest_promotion::build_plan(root, &args(root))
            .err()
            .unwrap();
        // The compiled producer closure includes the wrapper. Its approval check refuses
        // before obligation verdicts; the later protected tip cannot approve the old producer.
        assert_eq!(
            refusal,
            "producer closure is not approved on protected staging: crates"
        );
        assert_eq!(snapshot(root), before);
    }
    #[test]
    fn ma6_apply_refuses_captured_route_drift_and_production_cannot_admit_depth() {
        let (f, host) = admitted_ma5(true);
        let root = f.root.path();
        let _installed = f.install();
        let _reader = host.install();
        let before = snapshot(root);
        assert!(!root
            .join("cli_manifests/depth_fixture/versions/0.2.0.json")
            .exists());
        let plan = manifest_promotion::build_plan(root, &args(root)).unwrap();
        f.api.put(
            "repos/fixture/isolated",
            json!({"id":999,"owner":{"id":20},"full_name":"fixture/isolated"}),
        );
        assert!(plan.apply(root).unwrap_err().contains("isolated"));
        assert_eq!(snapshot(root), before);
        f.host();
        // Planning kept the new metadata in scratch. Propose the actual pointer advance
        // directly to the production predicate; it must still refuse without verdicts.
        let pointer = "cli_manifests/depth_fixture/latest_validated.txt";
        assert_eq!(fs::read_to_string(root.join(pointer)).unwrap(), "0.1.0\n");
        let production =
            crate::agent_maintenance::closeout_proposal::Proposal::capture(root).unwrap();
        let err = production
            .plan_changes(
                root,
                vec![(pointer.into(), Some(b"0.2.0\n".to_vec()))],
                crate::depth_acceptance::plan_promotion,
            )
            .unwrap_err();
        assert!(err.contains("not independently satisfied"), "{err}");
        assert_eq!(snapshot(root), before);
    }
}
