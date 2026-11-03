//! MA1 preparation and MA2 re-freeze through the public prepare route under an isolated context.
use super::*;
use crate::agent_maintenance::{
    contract_policy::{dispatch_kind_str, dispatch_workflow_value, opened_from_path},
    prepare::{apply_routed, build_routed, Args, PreparePlan},
};
use crate::depth_record::DepthRecord;

const TARGET: &str = "x86_64-unknown-linux-gnu";
const GOVERNING: &str = "tests/fixtures/depth-synthetic/governing";

fn repo() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy(source: &Path, destination: &Path) {
    if source.is_dir() {
        for e in fs::read_dir(source).unwrap() {
            let e = e.unwrap();
            copy(&e.path(), &destination.join(e.file_name()));
        }
    } else {
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }
}

/// The isolated workspace: canonical publication inputs plus the synthetic governing templates.
fn workspace() -> (Fixture, Args) {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    for path in [
        "Cargo.toml",
        "crates/agent_api/Cargo.toml",
        "crates/agent_api/src",
        "cli_manifests",
        "docs/specs/unified-agent-api",
    ] {
        copy(&repo().join(path), &root.join(path));
    }
    let templates = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOVERNING);
    copy(
        &templates.join("manifest-root"),
        &root.join("cli_manifests/depth_fixture"),
    );
    let rules = root.join("cli_manifests/depth_fixture/RULES.json");
    let filled = fs::read_to_string(&rules)
        .unwrap()
        .replace("{{ISOLATED_OWNER}}", "fixture")
        .replace("{{ISOLATED_REPO}}", "isolated");
    fs::write(&rules, filled).unwrap();
    let registry = AgentRegistry::parse_unenforced(
        &fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap(),
    )
    .unwrap();
    let release = registry
        .find("depth_fixture")
        .unwrap()
        .maintenance
        .release_watch
        .clone()
        .unwrap();
    let workflow = dispatch_workflow_value("depth_fixture", &release).unwrap();
    let args = Args {
        agent: "depth_fixture".into(),
        current_version: "0.1.0".into(),
        latest_stable: "0.3.0".into(),
        target_version: "0.2.0".into(),
        opened_from: opened_from_path(&workflow).into(),
        detected_by: "fixture".into(),
        dispatch_kind: dispatch_kind_str(release.dispatch_kind).into(),
        dispatch_workflow: Some(workflow),
        branch_name: "automation/depth_fixture-maintenance-0.2.0".into(),
        request_recorded_at: "2026-11-03T00:00:00Z".into(),
        request_commit: "abcdef0".into(),
        dry_run: false,
        write: true,
    };
    copy(
        &repo().join(&args.opened_from),
        &root.join(&args.opened_from),
    );
    baseline(&f);
    (f, args)
}

/// The protected baseline: `0.1.0` is acquired and promoted by the real promotion route before
/// the depth table is installed, since promotion (MA6, B.6) still loads the production registry.
fn baseline(f: &Fixture) {
    let root = f.root.path();
    let declared = fs::read_to_string(root.join(REGISTRY_RELATIVE_PATH)).unwrap();
    f.registry(|doc| {
        let entry = doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        entry.remove("support_depth").unwrap();
    });
    acquire(root, "0.1.0");
    f.commit();
    let tip = git(root, &["rev-parse", "HEAD"]);
    git(root, &["update-ref", "refs/remotes/origin/staging", &tip]);
    let promotion = crate::manifest_promotion::build_plan(
        root,
        &crate::manifest_promotion::Args {
            agent: "depth_fixture".into(),
            version: "0.1.0".into(),
            passed_targets: vec![TARGET.into()],
            validation_commit: tip,
            write: true,
        },
    )
    .unwrap();
    promotion.apply(root).unwrap();
    write(root, REGISTRY_RELATIVE_PATH, declared);
    f.commit();
}

fn route(f: &Fixture) -> Route {
    route_in(f.root.path(), Some(f.profile()), f.api.clone())
        .unwrap()
        .expect("an installed profile resolves")
}

fn record(plan: &PreparePlan) -> DepthRecord {
    let file = plan
        .files
        .iter()
        .find(|f| f.relative_path.ends_with("/depth-record.json"))
        .expect("the plan carries the depth record");
    DepthRecord::parse(
        std::str::from_utf8(&file.contents).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap()
}

/// Acquisition writes for one release: the per-target shard of the fixture CLI's surface, the
/// union, the wrapper coverage and the reports.
fn acquire(root: &Path, version: &str) {
    let base = root.join("cli_manifests/depth_fixture");
    let shard = json!({"snapshot_schema_version":1,"tool":"depth-fixture-cli",
        "collected_at":"2026-11-03T00:00:00Z","binary":{"sha256":"a".repeat(64),"size_bytes":1,
        "platform":{"os":"linux","arch":"x86_64"},"target_triple":TARGET,
        "version_output":format!("depth_fixture {version}"),"semantic_version":version},
        "commands":[
        {"path":[],"about":"synthetic CLI for the support-depth machinery proof","flags":[
            {"long":"--help","short":"-h","takes_value":false},
            {"long":"--surface-json","takes_value":false},
            {"long":"--version","short":"-V","takes_value":false}]},
        {"path":["run"],"about":"Run one prompt","usage":"depth_fixture run [OPTIONS] [--] <PROMPT>",
            "args":[{"name":"PROMPT","required":true,"variadic":false}],
            "flags":[{"long":"--help","short":"-h","takes_value":false},
            {"long":"--json","takes_value":false},
            {"long":"--model","takes_value":true,"value_name":"NAME"}]}]});
    write(
        root,
        &format!("cli_manifests/depth_fixture/snapshots/{version}/{TARGET}.json"),
        serde_json::to_vec_pretty(&shard).unwrap(),
    );
    let coverage = depth_fixture::wrapper_coverage_manifest::wrapper_coverage_manifest();
    write(
        root,
        "cli_manifests/depth_fixture/wrapper_coverage.json",
        serde_json::to_vec_pretty(&coverage).unwrap(),
    );
    crate::manifest_union::run(crate::manifest_union::Args {
        root: Some(base.clone()),
        rules: None,
        version: version.into(),
    })
    .unwrap();
    crate::manifest_report::run(crate::manifest_report::Args {
        root: Some(base),
        rules: None,
        version: version.into(),
    })
    .unwrap();
}

#[test]
fn the_public_prepare_route_freezes_p_then_o_under_the_isolated_context() {
    let (f, args) = workspace();
    let root = f.root.path();
    // The ordinary route keeps its production refusal of the depth-carrying registry.
    assert!(build_routed(root, &args, None).is_err());

    // MA1: P only, since nothing is acquired yet.
    let p = build_routed(root, &args, Some(&route(&f))).unwrap();
    assert!(record(&p).obligations.is_none());
    assert!(apply_routed(root, &p, None).is_err());
    apply_routed(root, &p, Some(&route(&f))).unwrap();
    f.commit();

    // MA2: acquisition, then the re-freeze adds O under the same P.
    acquire(root, "0.2.0");
    f.commit();
    let o = build_routed(root, &args, Some(&route(&f))).unwrap();
    let frozen = record(&o);
    assert_eq!(frozen.policy, record(&p).policy);
    assert_eq!(frozen.obligations.as_ref().unwrap().targets, [TARGET]);
    // Honest O: no classification is approved in P yet, so every owned `run` surface stays
    // classification_required, and the waived `run --help` is carried required work. Neither is
    // cleared to make the chain look ready.
    let o_part = frozen.obligations.as_ref().unwrap();
    let unresolved: Vec<_> = o_part
        .classification_required
        .iter()
        .map(|c| c.surface.surface_id.as_str())
        .collect();
    assert_eq!(unresolved, ["--help", "--json", "--model", "PROMPT", "run"]);
    let work: Vec<_> = o_part
        .acceptance_work
        .iter()
        .map(|w| w.surface.surface_id.as_str())
        .collect();
    assert_eq!(work, ["--help"]);
    apply_routed(root, &o, Some(&route(&f))).unwrap();
    f.commit();
    let again = build_routed(root, &args, Some(&route(&f))).unwrap();
    assert_eq!(again.files, o.files);
    assert_eq!(
        apply_routed(root, &again, Some(&route(&f)))
            .unwrap()
            .written,
        0
    );
}

#[test]
fn a_plan_never_applies_after_the_protected_tip_moves() {
    let (f, args) = workspace();
    let root = f.root.path();
    let plan = build_routed(root, &args, Some(&route(&f))).unwrap();
    write(root, "docs/input.txt", "moved\n");
    f.commit();
    let error = apply_routed(root, &plan, Some(&route(&f)))
        .err()
        .unwrap()
        .to_string();
    assert!(
        error.contains("different route or protected tip"),
        "{error}"
    );
    assert!(!root
        .join("cli_manifests/depth_fixture/reports/0.2.0/depth-record.json")
        .exists());
}

#[test]
fn only_an_uninstalled_profile_yields_the_ordinary_route() {
    let f = Fixture::new_for(&scope::SYNTHETIC);
    let root = f.root.path();
    assert!(route_in(root, None, f.api.clone()).unwrap().is_none());
    assert!(route_in(root, Some(f.profile()), f.api.clone())
        .unwrap()
        .is_some());
    // A failing installed profile errors; it never falls back to the ordinary route.
    f.api.put(
        "repos/fixture/isolated",
        json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
    );
    assert!(route_in(root, Some(f.profile()), f.api.clone()).is_err());
}
