//! MA4 evidence under the isolated context (B.4 part 2): the guarded host authenticates the
//! population and stages a bundle outside the candidate; the executor runs the offline command
//! in the candidate; the relay delivers only reporting E; the audit re-authenticates it.
use super::ma4_route::{run, run_as};
use super::prepare::{acquire, workspace};
use super::*;
use crate::agent_lifecycle::maintenance_request_path;
use crate::agent_maintenance::{
    execute,
    prepare::{apply_prepare_plan, build_prepare_plan},
};
use crate::depth_ci::profile::Profile;
use crate::depth_record::{DepthRecord, DepthResult};

const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const EVIDENCE: &str = "cli_manifests/depth_fixture/reports/0.2.0/evidence/synthetic-wrapper-v1";
const RUNS: &str = "actions/workflows/ci.yml/runs?created=%3E%3D2026-11-03&per_page=100&page=1";
type HostEdit = Box<dyn Fn(&crate::depth_evidence::SyntheticHost)>;
const ENTRY: &str = "isolated_maintenance::tests::ma4_evidence::executor_entry";

/// The executor's real work in the connected chain: the offline command, in its own process.
#[test]
#[ignore = "subprocess entry for the MA4 executor of the connected chain; a no-op elsewhere"]
fn executor_entry() {
    if std::env::var_os("XTASK_AGENT_MAINTENANCE_RUN_ID").is_none() {
        return;
    }
    let root = std::env::current_dir().unwrap();
    match crate::depth_evidence::assemble_staged(&root) {
        Ok(summary) => print!("{summary}"),
        Err(reason) => panic!("depth-evidence-assemble refused: {reason}"),
    }
}

/// A stand-in executor outside the workspace. It answers the preflight; otherwise it records the
/// channel locator it was given, runs the offline command, then runs `extra`.
fn executor(dir: &Path, extra: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(format!("executor-{}.sh", extra.len()));
    let exe = std::env::current_exe().unwrap();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprompt=$(cat)\ncase \"$prompt\" in\n*'Repository preflight'*) echo {} ;;\n*) echo \"$XTASK_AGENT_MAINTENANCE_EVIDENCE_CHANNEL\" > {}\n\"{}\" --exact {ENTRY} --ignored --quiet || exit 1\n{extra}\n;;\nesac\n",
            execute::PREFLIGHT_SENTINEL,
            dir.join("locator").display(),
            exe.display(),
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_string_lossy().into_owned()
}

/// MA1, acquisition and MA2's re-freeze; then the isolated repository's protected producer
/// closure for that exact frozen generation, with one faithful, adopted CI run of it.
fn chain() -> (Fixture, crate::depth_evidence::SyntheticHost) {
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
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
    }
    // The CI producer runs over this exact frozen generation (its profile fixture record).
    write(
        root,
        Profile::Synthetic.fixture(),
        std::fs::read(root.join(RECORD)).unwrap(),
    );
    let host = crate::depth_evidence::synthetic_host(root);
    f.host();
    (f, host)
}

fn record(root: &Path) -> DepthRecord {
    DepthRecord::parse(
        &std::fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap()
}
/// The audit's own evaluation: it re-authenticates the full population, whatever E claims.
fn audited(root: &Path) -> Vec<DepthResult> {
    let record = record(root);
    let results = crate::depth_evidence::evaluate_isolated_record(root, &record).unwrap();
    Profile::Synthetic
        .obligations(&record)
        .iter()
        .map(|o| results.result(&record, o))
        .collect()
}
fn tracked(f: &Fixture) -> String {
    let root = f.root.path();
    format!(
        "{}\n{}",
        git(root, &["rev-parse", "HEAD"]),
        git(root, &["status", "--porcelain=v1", "--untracked-files=all"])
    )
}

#[test]
fn ma4_writes_authenticated_reporting_e_that_the_audit_reevaluates() {
    let (f, host) = chain();
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let binary = executor(bin.path(), "");
    let frozen = record(root);
    assert!(frozen.execution.is_none());
    // Without the route, MA4 keeps the production refusal and nothing is staged.
    let refused = run(root, &binary, false, "ma4-evidence").unwrap_err();
    assert!(refused.contains("support_depth"), "{refused}");

    let _installed = f.install();
    let _reader = host.install();
    {
        let _gates = execute::gate_stub::stub();
        let written = run_as(root, &binary, "ma4-evidence").unwrap();
        assert!(written.contains("depth_fixture"), "{written}");
    }
    // Only reporting E reached the caller: Event/P/O/entries are the frozen ones.
    let delivered = record(root);
    let e = delivered.execution.clone().expect("delivered E");
    assert_eq!(
        (&delivered.event, &delivered.policy, &delivered.obligations),
        (&frozen.event, &frozen.policy, &frozen.obligations)
    );
    assert_eq!(delivered.acceptance_entries, frozen.acceptance_entries);
    assert_eq!(e.results.len(), 11);
    for reference in e.results.iter().flat_map(|r| &r.evidence) {
        assert!(reference.path.starts_with(EVIDENCE), "{}", reference.path);
        assert!(root.join(&reference.path).is_file(), "{}", reference.path);
    }
    // The invocation's channel and bundle ended with it; its locator is not a route.
    let locator = PathBuf::from(
        std::fs::read_to_string(bin.path().join("locator"))
            .unwrap()
            .trim(),
    );
    assert!(!locator.parent().unwrap().exists(), "{}", locator.display());
    f.commit();
    host.staging(&git(root, &["rev-parse", "HEAD"]));
    assert_eq!(audited(root), vec![DepthResult::Verified; 11]);

    // A standalone run in a copy of the workspace, with or without the old locator, refuses.
    let copy = tempfile::tempdir().unwrap();
    let clone = copy.path().join("candidate");
    git(
        root,
        &[
            "clone",
            "--quiet",
            root.to_str().unwrap(),
            clone.to_str().unwrap(),
        ],
    );
    for locator in [None, Some(locator)] {
        let error = crate::depth_evidence::staged_assemble(&clone, locator).unwrap_err();
        assert!(error.contains("live"), "{error}");
    }
    assert_eq!(git(&clone, &["status", "--porcelain"]), "");
}

#[test]
fn an_earlier_bundle_never_outlives_a_changed_population() {
    let (f, host) = chain();
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let binary = executor(bin.path(), "");
    let _installed = f.install();
    let _reader = host.install();
    {
        let _gates = execute::gate_stub::stub();
        run_as(root, &binary, "ma4-evidence").unwrap();
    }
    f.commit();
    host.staging(&git(root, &["rev-parse", "HEAD"]));
    assert_eq!(audited(root), vec![DepthResult::Verified; 11]);
    let unverified = vec![DepthResult::Unverified; 11];
    // An omitted nonterminal or failed attempt, or a revoked adoption, leaves the delivered E
    // unverified: the audit re-authenticates the whole population, not the earlier bundle.
    let edits: Vec<HostEdit> = vec![
        Box::new(|h| h.add_attempt(RUNS, |job| job["status"] = json!("in_progress"))),
        Box::new(|h| {
            h.add_attempt(RUNS, |job| {
                job["steps"][0]["conclusion"] = json!("failure");
                job["conclusion"] = json!("failure");
            })
        }),
        Box::new(|h| h.revoke_adoption()),
    ];
    for edit in edits {
        let changed = host.fork();
        edit(&changed);
        let _changed = changed.install();
        assert_eq!(audited(root), unverified);
        // A later MA4 run under the changed host stages no evidence, so the executor's command
        // refuses and the caller keeps the earlier record.
        let before = tracked(&f);
        let delivered = std::fs::read(root.join(RECORD)).unwrap();
        let _gates = execute::gate_stub::stub();
        let refused = run_as(root, &binary, "ma4-evidence-changed").unwrap_err();
        assert!(refused.contains("rejected candidate retained"), "{refused}");
        // The test harness standing in for the command reports its panic on stdout.
        let output = std::fs::read_to_string(root.join(
            "docs/agents/.uaa-temp/agent-maintenance/runs/ma4-evidence-changed/codex-stdout.log",
        ))
        .unwrap();
        assert!(
            output.contains("no authenticated evidence was staged"),
            "{output}"
        );
        assert_eq!(std::fs::read(root.join(RECORD)).unwrap(), delivered);
        assert_eq!(tracked(&f).lines().next(), before.lines().next());
        git(root, &["clean", "-fdxq", "docs"]);
    }
}

#[test]
fn forbidden_writes_and_route_or_tip_drift_preserve_the_caller() {
    let (f, host) = chain();
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let _installed = f.install();
    let _reader = host.install();
    let before = std::fs::read(root.join(RECORD)).unwrap();
    // The executor writes E but also changes the request: the relay refuses everything.
    let request = maintenance_request_path("depth_fixture");
    let forbidden = executor(bin.path(), &format!("echo '# edit' >> {request}"));
    {
        let _gates = execute::gate_stub::stub();
        let refused = run_as(root, &forbidden, "ma4-evidence-forbidden").unwrap_err();
        assert!(refused.contains("protected request"), "{refused}");
    }
    assert_eq!(std::fs::read(root.join(RECORD)).unwrap(), before);
    git(root, &["clean", "-fdxq", "docs"]);
    // The isolated identity or protected tip moves while the executor runs: delivery refuses.
    let binary = executor(bin.path(), "");
    let moved = git(root, &["rev-parse", "HEAD~1"]);
    for (endpoint, value) in [
        (
            "repos/fixture/isolated",
            json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
        ),
        (
            "repos/fixture/isolated/branches/staging",
            json!({"name":"staging","protected":true,"commit":{"sha":moved}}),
        ),
    ] {
        let api = f.api.clone();
        let _gates = execute::gate_stub::stub_with(move || api.put(endpoint, value.clone()));
        let refused = run_as(root, &binary, "ma4-evidence-moved").unwrap_err();
        assert!(
            refused.contains("isolated maintenance route changed"),
            "{refused}"
        );
        assert_eq!(std::fs::read(root.join(RECORD)).unwrap(), before);
        git(root, &["clean", "-fdxq", "docs"]);
        f.host();
    }
}
