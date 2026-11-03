//! MA1 preparation and MA2 re-freeze through the public prepare route under an isolated context.
use super::*;
use crate::agent_maintenance::{
    contract_policy::{dispatch_kind_str, dispatch_workflow_value, opened_from_path},
    prepare::{
        apply_prepare_plan, apply_routed, build_prepare_plan, build_routed, Args, PreparePlan,
    },
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
    // Without an installed profile the exported entrypoints keep their production refusal.
    assert!(build_prepare_plan(root, &args).is_err());
    let _installed = f.install();

    // MA1 through the exported builder and apply: P only, since nothing is acquired yet.
    let p = build_prepare_plan(root, &args).unwrap();
    assert!(record(&p).obligations.is_none());
    assert!(apply_routed(root, &p, None).is_err());
    apply_prepare_plan(root, &p).unwrap();
    f.commit();

    // MA2: acquisition, then the re-freeze adds O under the same P.
    acquire(root, "0.2.0");
    f.commit();
    let o = build_prepare_plan(root, &args).unwrap();
    let frozen = record(&o);
    assert_eq!(frozen.policy, record(&p).policy);
    assert_eq!(frozen.obligations.as_ref().unwrap().targets, [TARGET]);
    // Honest O: no classification is approved in P yet, so every owned `run` surface stays
    // classification_required. `run --help` is a parity exclusion, so no waiver is carried work.
    let o_part = frozen.obligations.as_ref().unwrap();
    let unresolved: Vec<_> = o_part
        .classification_required
        .iter()
        .map(|c| c.surface.surface_id.as_str())
        .collect();
    assert_eq!(unresolved, ["--help", "--json", "--model", "PROMPT", "run"]);
    assert!(
        o_part.acceptance_work.is_empty(),
        "{:?}",
        o_part.acceptance_work
    );
    apply_prepare_plan(root, &o).unwrap();
    f.commit();
    let again = build_prepare_plan(root, &args).unwrap();
    assert_eq!(again.files, o.files);
    assert_eq!(apply_prepare_plan(root, &again).unwrap().written, 0);

    // The maintainer reviews the reported basis and approves CORE rows in P. P changes only in a
    // new generation, so the rows enter under a new Event.
    let [(operation, basis)] = o.classification_bases.as_slice() else {
        panic!("one basis: {:?}", o.classification_bases)
    };
    assert_eq!(operation, "run");
    let surfaces: Vec<_> = o_part
        .classification_required
        .iter()
        .map(|c| c.surface.clone())
        .collect();
    let next = Args {
        request_recorded_at: "2026-11-04T00:00:00Z".into(),
        request_commit: "abcdef1".into(),
        ..args.clone()
    };
    // A stale basis refuses rather than reusing the assignments.
    classify(&f, &surfaces, &"0".repeat(64));
    f.commit();
    let stale = build_prepare_plan(root, &next).err().unwrap().to_string();
    assert!(stale.contains("stale classification_basis"), "{stale}");
    classify(&f, &surfaces, basis);
    f.commit();
    let classified = build_prepare_plan(root, &next).unwrap();
    let ready = record(&classified);
    assert_ne!(ready.policy, frozen.policy);
    let ready_o = ready.obligations.as_ref().unwrap();
    assert!(ready_o.classification_required.is_empty());
    assert!(ready_o.acceptance_work.is_empty());
    apply_prepare_plan(root, &classified).unwrap();
}

const RATIONALE: &str =
    "Part of the wrapper's whole `run --json` surface; CORE covers its complete \
    value, default and effect regime, and it adds no effect beyond `run`.";

/// The approved synthetic classification (B.3): every owned `run` unit selects only CORE.
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
        // Inline rows: a fresh array of tables would render after the last `[[agents]]` entry.
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
            row.insert("rationale", RATIONALE.into());
            rows.push(row);
        }
        operation["classification_basis"] = toml_edit::value(basis);
        operation["classifications"] = toml_edit::value(rows);
    });
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
    assert!(error.contains("different route binding"), "{error}");
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

#[test]
fn a_plan_never_applies_to_another_candidate() {
    let (f, args) = workspace();
    let root = f.root.path();
    let plan = build_routed(root, &args, Some(&route(&f))).unwrap();
    // The candidate HEAD moves with an identical tree; protected staging does not.
    let tip = git(root, &["rev-parse", "HEAD"]);
    git(
        root,
        &["commit", "-q", "--allow-empty", "-m", "candidate moves"],
    );
    let head = git(root, &["rev-parse", "HEAD"]);
    f.api.put(
        &format!("repos/fixture/isolated/compare/{tip}...{head}"),
        json!({"status":"ahead","merge_base_commit":{"sha":tip}}),
    );
    f.api.put(
        &format!("repos/fixture/isolated/compare/{}...{head}", f.baseline),
        json!({"status":"ahead","merge_base_commit":{"sha":f.baseline}}),
    );
    // The moved candidate is published to the isolated repository.
    f.api.put(
        &format!("repos/fixture/isolated/commits/{head}"),
        json!({"sha":head}),
    );
    let moved = route(&f);
    let error = apply_routed(root, &plan, Some(&moved))
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("different route binding"), "{error}");
}
