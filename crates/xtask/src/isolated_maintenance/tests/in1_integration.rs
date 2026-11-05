//! CP-P1: binding comes only from authenticated T, checkout H and Git parentage.
use super::*;
use crate::depth_acceptance::check_integration;

mod regressions;

// Moved from tests.rs to keep its module declarations within the code-line cap.
#[test]
fn matching_authenticated_private_scope_resolves_without_admission() {
    let f = Fixture::new();
    f.resolve().unwrap().recheck(f.root.path()).unwrap();
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    head: String,
    status: String,
    index: Vec<u8>,
    files: Vec<(String, Vec<u8>)>,
}
fn snapshot(root: &Path) -> Snapshot {
    Snapshot {
        head: git(root, &["rev-parse", "HEAD"]),
        status: git(root, &["status", "--porcelain=v1", "--untracked-files=all"]),
        index: fs::read(root.join(".git/index")).unwrap(),
        files: git(root, &["ls-files"])
            .lines()
            .map(|p| (p.into(), fs::read(root.join(p)).unwrap()))
            .collect(),
    }
}
fn head(f: &Fixture) -> String {
    git(f.root.path(), &["rev-parse", "HEAD"])
}
fn local(f: &Fixture, tip: &str) {
    git(
        f.root.path(),
        &["update-ref", "refs/remotes/origin/staging", tip],
    );
}
/// Commit a candidate without pushing the fake protected tip. Host comparisons deliberately
/// claim ancestry even in negative tests: IN1 must independently check the local Git objects.
fn candidate(f: &Fixture, tip: &str) -> String {
    git(f.root.path(), &["add", "."]);
    git(
        f.root.path(),
        &["commit", "--allow-empty", "-m", "IN1 candidate"],
    );
    let head = head(f);
    f.api.put(
        &format!("repos/fixture/isolated/compare/{}...{head}", f.baseline),
        json!({"status":"ahead","merge_base_commit":{"sha":f.baseline}}),
    );
    f.api.put(
        &format!("repos/fixture/isolated/compare/{tip}...{head}"),
        json!({"status":"ahead","merge_base_commit":{"sha":tip}}),
    );
    f.api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    head
}
fn refused(f: &Fixture, parent: &str) -> String {
    let before = snapshot(f.root.path());
    let err = check_integration(f.root.path(), parent).unwrap_err();
    assert!(err.contains("isolated IN1 binding"), "{err}");
    assert_eq!(before, snapshot(f.root.path()));
    err
}
#[test]
fn in1_refuses_equal_head_and_parent_even_at_authenticated_tip() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    local(&f, &tip);
    refused(&f, &tip);
}
#[test]
fn in1_admission_and_observation_of_ordinary_transition() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    local(&f, &tip);
    let next = candidate(&f, &tip);
    check_integration(f.root.path(), &tip).unwrap();
    f.host();
    local(&f, &next);
    check_integration(f.root.path(), &tip).unwrap();
}

#[test]
fn in1_refuses_old_or_stacked_parent_and_non_tip_observation() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    local(&f, &tip);
    let stacked = candidate(&f, &tip);
    candidate(&f, &tip);
    // Admission allows multiple candidate commits when P is the authenticated tip.
    check_integration(f.root.path(), &tip).unwrap();
    // P is a real ancestor of H, but is another queued entry rather than authenticated T.
    refused(&f, &stacked);
    // H = P also refuses away from T.
    refused(&f, &head(&f));
    // The old base is also refused once T has advanced past it.
    f.host();
    let now = head(&f);
    local(&f, &now);
    refused(&f, &tip);
    check_integration(f.root.path(), &stacked).unwrap();
}
#[test]
fn in1_refuses_forged_parent_and_non_descendant_despite_host_ancestry_claim() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let base = head(&f);
    let tip = candidate(&f, &base);
    f.host();
    local(&f, &tip);
    // A sibling commit does not descend from T. The fake host's assertion is insufficient.
    git(f.root.path(), &["reset", "--hard", &base]);
    write(f.root.path(), "sibling.txt", "unrelated candidate\n");
    let sibling = candidate(&f, &tip);
    assert_ne!(sibling, tip);
    let err = refused(&f, &tip);
    assert!(
        err.contains("does not descend from the authenticated tip"),
        "{err}"
    );
    git(f.root.path(), &["reset", "--hard", &tip]);
    refused(&f, &sibling);
    refused(&f, &"f".repeat(40));
}
#[test]
fn in1_merge_observation_uses_only_first_parent_and_refuses_chain() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let base = head(&f);
    let first = candidate(&f, &base);
    git(f.root.path(), &["reset", "--hard", &base]);
    // Distinct trees make these distinct siblings even if both commits share a timestamp.
    write(f.root.path(), "sibling.txt", "second parent\n");
    let second = candidate(&f, &base);
    assert_ne!(first, second);
    let tree = git(f.root.path(), &["rev-parse", "HEAD^{tree}"]);
    let merged = git(
        f.root.path(),
        &[
            "commit-tree",
            &tree,
            "-p",
            &first,
            "-p",
            &second,
            "-m",
            "IN1 merge",
        ],
    );
    git(f.root.path(), &["reset", "--hard", &merged]);
    f.host();
    local(&f, &merged);
    check_integration(f.root.path(), &first).unwrap();
    refused(&f, &second);
    refused(&f, &base);
}
#[test]
fn in1_refuses_local_tip_mismatch_and_failing_installed_route() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    let next = candidate(&f, &tip);
    local(&f, &next);
    refused(&f, &tip);
    f.api.put("repos/fixture/isolated", json!({"id":999}));
    // Even an empty root/record set cannot narrow away the failed repository-wide route.
    assert!(check_integration(f.root.path(), &tip)
        .unwrap_err()
        .contains("repository id"));
}
#[test]
fn in1_refuses_tip_moved_after_authentication() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let _installed = f.install();
    let tip = head(&f);
    local(&f, &tip);
    let next = candidate(&f, &tip);
    let endpoint = "repos/fixture/isolated/branches/staging";
    let stable = json!({"name":"staging","protected":true,"commit":{"sha":tip}});
    let moved = json!({"name":"staging","protected":true,"commit":{"sha":next}});
    // Resolve authenticates T (four reads); capture rechecks it (six); the initial binding
    // rechecks before and after (four). The closing binding then observes the moved tip.
    f.api.1.lock().unwrap().insert(
        endpoint.into(),
        std::iter::repeat(stable).take(14).chain([moved]).collect(),
    );
    let err = check_integration(f.root.path(), &tip).unwrap_err();
    assert!(err.contains("isolated"), "{err}");
    assert!(f.api.1.lock().unwrap()[endpoint].is_empty());
}
#[test]
fn in1_production_keeps_equal_revision_and_ordinary_transition_semantics() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    f.registry(|doc| {
        let agents = doc["agents"].as_array_of_tables_mut().unwrap();
        agents
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap()
            .remove("support_depth");
    });
    f.commit();
    let tip = head(&f);
    assert!(route(f.root.path()).unwrap().is_none());
    // Production never required T, a local integration ref, or H != P.
    check_integration(f.root.path(), &tip).unwrap();
    let stacked = candidate(&f, &tip);
    candidate(&f, &tip);
    check_integration(f.root.path(), &tip).unwrap();
    check_integration(f.root.path(), &stacked).unwrap();
}

#[test]
fn in1_per_root_tip_validation_uses_the_routed_registry_and_comparison() {
    use crate::agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan};
    let (f, args) = super::prepare::workspace();
    let root = f.root.path();
    let _installed = f.install();
    apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
    f.commit();
    let parent = head(&f);
    local(&f, &parent);
    candidate(&f, &parent);
    let before = snapshot(root);
    let route = route(root).unwrap().unwrap();
    assert!(route
        .integration_registry(root, &parent)
        .unwrap()
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
    let comparison = route
        .integration_comparison(root, &parent, "depth_fixture")
        .unwrap();
    assert!(comparison.effects.is_empty());
    assert!(comparison.missing_entries.is_empty());
    assert!(comparison.input_issues.is_empty());
    assert_eq!(
        crate::manifest_validate::run(crate::manifest_validate::Args {
            root: Some(root.join("cli_manifests/depth_fixture")),
            rules: None,
            schema: None,
            version_schema: None,
            mode: crate::manifest_validate::Mode::Check,
            json: true,
            tip: Some(parent),
        }),
        0
    );
    assert_eq!(snapshot(root), before);
}

// These fixtures mirror B.5b's real MA4 delivery and both MA5 writers. Keep the socket-bound
// chain separate from the binding-only tests so those can run in restricted sandboxes.
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
        super::super::publication::refresh(&f, &host);
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
    fn validate(root: &Path, parent: &str) -> i32 {
        super::super::publication::validate_candidate(root, parent)
    }
    fn admit_and_observe(writer: fn(&Path) -> Result<(), String>, authored: bool) {
        let (f, host) = ready();
        let root = f.root.path();
        let installed = f.install();
        let _reader = host.install();
        // The closeout writer takes a maintainer-authored closeout; prepare authors its own.
        if authored {
            author(&f, &host);
        }
        let parent = head(&f);
        let reporting = record(root);
        assert!(reporting.acceptance_entries.is_empty());
        writer(root).unwrap();
        super::super::publication::assert_qualified(root);
        let closed = record(root);
        assert_eq!(
            closed.stated_sha256().unwrap(),
            reporting.stated_sha256().unwrap()
        );
        assert!(matches!(
            closed.acceptance_entries.as_slice(),
            [AcceptanceEntry::MaintenanceCloseout { .. }]
        ));
        let next = candidate(&f, &parent);
        let before = snapshot(root);
        check_integration(root, &parent).unwrap();
        // The per-root --tip path must also read both registries and compare under the route.
        assert_eq!(validate(root, &parent), 0);
        assert_eq!(snapshot(root), before);
        f.host();
        host.staging(&next);
        local(&f, &next);
        check_integration(root, &parent).unwrap();
        assert_eq!(validate(root, &parent), 0);
        assert_eq!(snapshot(root), before);
        drop(installed);
        let err = check_integration(root, &parent).unwrap_err();
        assert!(err.contains("not independently satisfied"), "{err}");
        assert_eq!(snapshot(root), before);
    }
    #[test]
    fn in1_closeout_candidate_admission_and_post_push_observation() {
        admit_and_observe(close, true);
    }
    #[test]
    fn in1_prepared_closeout_candidate_admission_and_post_push_observation() {
        admit_and_observe(prepare, false);
    }
    #[test]
    fn in1_refuses_forged_closeout_entries_in_admission_and_observation() {
        let (f, host) = ready();
        let root = f.root.path();
        let _installed = f.install();
        let _reader = host.install();
        prepare(root).unwrap();
        let genuine = record(root);
        let closeout_bytes = fs::read(root.join(CLOSEOUT)).unwrap();
        let parent = head(&f);
        for forge in 0..3 {
            let mut changed = genuine.clone();
            let AcceptanceEntry::MaintenanceCloseout {
                closeout,
                record_sha256,
            } = &mut changed.acceptance_entries[0]
            else {
                panic!("closeout");
            };
            match forge {
                0 => closeout.sha256 = "0".repeat(64),
                1 => closeout.path = "docs/forged-closeout.json".into(),
                _ => *record_sha256 = "0".repeat(64),
            }
            write(root, RECORD, serde_json::to_vec_pretty(&changed).unwrap());
            write(root, CLOSEOUT, &closeout_bytes);
            let next = candidate(&f, &parent);
            let before = snapshot(root);
            for observe in [false, true] {
                if observe {
                    f.host();
                    host.staging(&next);
                    local(&f, &next);
                }
                let err = check_integration(root, &parent).unwrap_err();
                assert!(
                    err.contains("exact current entry")
                        || err.contains("describe the current record"),
                    "{err}"
                );
                assert_eq!(snapshot(root), before);
            }
            git(root, &["reset", "--hard", &parent]);
            f.host();
            host.staging(&parent);
            local(&f, &parent);
        }
    }
    #[test]
    fn in1_reauthenticates_population_in_admission_and_post_push_observation() {
        let (f, host) = ready();
        let root = f.root.path();
        let _installed = f.install();
        let _reader = host.install();
        prepare(root).unwrap();
        let closed = record(root);
        assert!(closed
            .execution
            .as_ref()
            .unwrap()
            .results
            .iter()
            .all(|r| r.result == DepthResult::Verified));
        let parent = head(&f);
        let next = candidate(&f, &parent);
        check_integration(root, &parent).unwrap();
        let crate::depth_record::Event::Maintenance {
            request_recorded_at,
            ..
        } = closed.event
        else {
            panic!("maintenance");
        };
        let runs = format!(
            "actions/workflows/ci.yml/runs?created=%3E%3D{}&per_page=100&page=1",
            &request_recorded_at[..10]
        );
        let before = snapshot(root);
        for observe in [false, true] {
            let _reader = host.install();
            if observe {
                f.host();
                host.staging(&next);
                local(&f, &next);
                check_integration(root, &parent).unwrap();
            }
            for edit in 0..3 {
                let changed = host.fork();
                match edit {
                    0 => changed.add_attempt(&runs, |job| job["status"] = json!("in_progress")),
                    1 => changed.add_attempt(&runs, |job| {
                        job["steps"][0]["conclusion"] = json!("failure");
                        job["conclusion"] = json!("failure");
                    }),
                    _ => changed.revoke_adoption(),
                }
                {
                    let _changed = changed.install();
                    let err = check_integration(root, &parent).unwrap_err();
                    assert!(
                        [
                            "not independently satisfied",
                            "missing/ambiguous attempt witness artifact",
                            "missing exact authenticated adequacy adoption"
                        ]
                        .iter()
                        .any(|why| err.contains(why)),
                        "{err}"
                    );
                }
                // The host override drops without restoring the previous reader.
                let _restored = host.install();
                check_integration(root, &parent).unwrap();
                assert_eq!(snapshot(root), before);
            }
        }
    }
}
