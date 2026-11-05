//! Roadmap B.7 under an installed isolated route: after MA1/MA2 prepare the synthetic subject
//! through the authenticated route (declared table, retained record, a depth-bearing support
//! bundle), the onboarding, neutral and lower writers (Annex A ON1–ON5, NE1, NE3, NE4, LW1) still
//! refuse and change nothing. The route is MA authority only; it grants these routes none. Routes
//! reachable only through the binary are driven in `tests/depth_refusal_routes.rs`.
use super::prepare::{acquire, workspace};
use super::*;
use crate::agent_maintenance::prepare::{apply_prepare_plan, build_prepare_plan};
use clap::Parser;
use sha2::{Digest, Sha256};

/// Workspace-scoped guards see the declaration first; manifest-scoped guards see the record first.
const REFUSED: [&str; 2] = [
    "unsupported depth write (declared enrollment)",
    "unsupported depth write (retained record)",
];
const APPROVAL: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml";
const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";

/// Each path's type, mode and digest of its bytes (or link target), as the integration helper.
type Tree = BTreeMap<std::path::PathBuf, (char, u32, String)>;

fn tree(root: &Path) -> Tree {
    use std::os::unix::fs::PermissionsExt;
    fn walk(root: &Path, dir: &Path, out: &mut Tree) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let meta = fs::symlink_metadata(&path).unwrap();
            let rel = path.strip_prefix(root).unwrap().to_owned();
            let mode = meta.permissions().mode();
            if meta.file_type().is_symlink() {
                let target = fs::read_link(&path).unwrap().display().to_string();
                out.insert(rel, ('l', mode, target));
            } else if meta.is_dir() {
                out.insert(rel, ('d', mode, String::new()));
                walk(root, &path, out);
            } else {
                let digest = hex::encode(Sha256::digest(fs::read(&path).unwrap()));
                out.insert(rel, ('f', mode, digest));
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn refused(route: &str, error: impl std::fmt::Debug) {
    let error = format!("{error:?}");
    assert!(
        REFUSED.iter().any(|reason| error.contains(reason)),
        "{route} did not refuse: {error}"
    );
}

fn repo_json<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn an_installed_route_grants_unsupported_writers_no_depth_authority() {
    let (f, args) = workspace();
    let root = f.root.path();
    let _installed = f.install();
    for acquired in [false, true] {
        if acquired {
            acquire(root, "0.2.0");
            f.commit();
        }
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
    }
    assert!(root.join(RECORD).is_file());
    // The route is live: it serves the scoped registry and a depth-bearing support bundle.
    let registry = crate::isolated_maintenance::registry(root).unwrap();
    assert!(registry
        .find("depth_fixture")
        .unwrap()
        .support_depth
        .is_some());
    let bundle = crate::support_matrix::generate_publication_bundle(root).unwrap();
    assert!(bundle.json.contains("depth_fixture"));
    let manifest = root.join("cli_manifests/depth_fixture");
    let before = tree(root);

    // ON1–ON5 library entrypoints (close-proving-run is binary-only).
    let out = &mut Vec::new();
    let onboard = crate::onboard_agent::Args::try_parse_from([
        "onboard-agent",
        "--write",
        "--approval",
        APPROVAL,
    ])
    .unwrap();
    refused(
        "ON1",
        crate::onboard_agent::run_in_workspace(root, onboard, out).unwrap_err(),
    );
    let follow_on = crate::runtime_follow_on::Args::try_parse_from([
        "runtime-follow-on",
        "--write",
        "--approval",
        APPROVAL,
        "--run-id",
        "depth-refusal",
    ])
    .unwrap();
    refused(
        "ON2 follow-on",
        crate::runtime_follow_on::run_in_workspace(root, follow_on, out).unwrap_err(),
    );
    let write = ["--write", "--approval", APPROVAL];
    let argv = |command: &'static str| std::iter::once(command).chain(write);
    let repair = crate::repair_runtime_evidence::Args::try_parse_from(argv("repair")).unwrap();
    refused(
        "ON2 repair",
        crate::repair_runtime_evidence::run_in_workspace(root, repair, out).unwrap_err(),
    );
    let prepare = crate::prepare_publication::Args::try_parse_from(argv("prepare")).unwrap();
    refused(
        "ON3",
        crate::prepare_publication::run_in_workspace(root, prepare, out).unwrap_err(),
    );
    let refresh = crate::publication_refresh::Args::try_parse_from(argv("refresh")).unwrap();
    refused(
        "ON4",
        crate::publication_refresh::run_in_workspace(root, refresh, out).unwrap_err(),
    );
    let closeout = crate::prepare_proving_run_closeout::Args::try_parse_from(argv("closeout"));
    refused(
        "ON5",
        crate::prepare_proving_run_closeout::run_in_workspace(root, closeout.unwrap(), out)
            .unwrap_err(),
    );

    // NE1: the route-generated bundle still cannot be delivered by the neutral writer.
    refused(
        "NE1",
        crate::support_matrix::write_publication_artifacts(root, &bundle).unwrap_err(),
    );
    // NE3 and NE4 on the subject's own manifest root.
    let metadata = crate::manifest_version_metadata::Args::try_parse_from([
        "manifest-version-metadata",
        "--root",
        manifest.to_str().unwrap(),
        "--version",
        "0.2.0",
        "--status",
        "validated",
        "--passed-target",
        "x86_64-unknown-linux-gnu",
    ])
    .unwrap();
    refused(
        "NE3",
        crate::manifest_version_metadata::run(metadata).unwrap_err(),
    );
    let fix = crate::manifest_validate::Args {
        root: Some(manifest.clone()),
        rules: None,
        schema: None,
        version_schema: None,
        mode: crate::manifest_validate::Mode::Fix,
        json: false,
        tip: None,
    };
    refused(
        "NE4",
        crate::manifest_validate::violation_codes(fix).unwrap_err(),
    );

    // LW1: the public lower boundaries.
    let pointer = crate::workspace_mutation::PlannedMutation::create(
        "cli_manifests/depth_fixture/latest_validated.txt.new",
        b"0.2.0\n".to_vec(),
    );
    refused(
        "LW1 batch",
        crate::workspace_mutation::apply_mutations(root, &[pointer]).unwrap_err(),
    );
    let state: crate::agent_lifecycle::LifecycleState =
        repo_json("docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json");
    let lifecycle = "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/state.json";
    refused(
        "LW1 lifecycle",
        crate::agent_lifecycle::write_lifecycle_state(root, lifecycle, &state).unwrap_err(),
    );
    let packet: crate::agent_lifecycle::PublicationReadyPacket = repo_json(
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json",
    );
    refused(
        "LW1 packet",
        crate::agent_lifecycle::write_publication_ready_packet(root, lifecycle, &packet)
            .unwrap_err(),
    );

    let after = tree(root);
    let changed: Vec<_> = before
        .iter()
        .filter(|(p, v)| after.get(*p) != Some(v))
        .map(|(p, _)| p)
        .chain(after.keys().filter(|p| !before.contains_key(*p)))
        .collect();
    assert!(changed.is_empty(), "refused routes changed {changed:?}");
}
