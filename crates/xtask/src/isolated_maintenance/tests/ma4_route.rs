//! MA4 under the isolated context (B.4): `execute-agent-maintenance` resolves the route once, in
//! the caller workspace, and every later request load uses the registry it captured.
use super::prepare::{acquire, workspace};
use super::*;
use crate::agent_lifecycle::maintenance_request_path;
use crate::agent_maintenance::{
    execute,
    prepare::{apply_prepare_plan, build_prepare_plan},
};

const LOG: &str = "docs/agents/lifecycle/depth_fixture-maintenance/governance/remediation-log.md";

/// A stand-in executor outside the workspace. It answers the read-only preflight; otherwise it
/// makes one packet-owned edit inside the writable surfaces, with no depth effect.
fn executor(dir: &Path) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("executor.sh");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nprompt=$(cat)\ncase \"$prompt\" in\n*'Repository preflight'*) echo {} ;;\n*) echo '- executor note' >> {LOG} ;;\nesac\n",
            execute::PREFLIGHT_SENTINEL
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_string_lossy().into_owned()
}

#[derive(Debug)]
pub(super) struct RunError {
    message: String,
    _recovery: Option<crate::candidate_execution::TestHolder>,
}
impl std::ops::Deref for RunError {
    type Target = str;
    fn deref(&self) -> &str {
        &self.message
    }
}
impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl RunError {
    fn new(message: String) -> Self {
        let recovery = message
            .rsplit_once("candidate retained at ")
            .map(|(_, path)| {
                let candidate = Path::new(path.trim());
                assert!(
                    candidate.is_dir(),
                    "reported recovery path must be retained"
                );
                crate::candidate_execution::TestHolder::new(candidate)
            });
        Self {
            message,
            _recovery: recovery,
        }
    }
}

fn dry_run(root: &Path, binary: &str) -> Result<String, RunError> {
    run(root, binary, false, "ma4-route")
}

/// A dry run and then a write run under one run id.
pub(super) fn run_as(root: &Path, binary: &str, run_id: &str) -> Result<String, RunError> {
    run(root, binary, false, run_id)?;
    run(root, binary, true, run_id)
}

pub(super) fn run(
    root: &Path,
    binary: &str,
    write: bool,
    run_id: &str,
) -> Result<String, RunError> {
    let mut out = Vec::new();
    execute::run_in_workspace(
        root,
        execute::Args {
            request: root.join(maintenance_request_path("depth_fixture")),
            dry_run: !write,
            write,
            run_id: Some(run_id.into()),
            codex_binary: Some(binary.into()),
        },
        &mut out,
    )
    .map(|()| String::from_utf8_lossy(&out).into_owned())
    .map_err(|e| RunError::new(e.to_string()))
}

#[test]
fn ma4_captures_the_scoped_registry_only_under_an_installed_route() {
    let (f, args) = workspace();
    let root = f.root.path();
    let bin = tempfile::tempdir().unwrap();
    let binary = executor(bin.path());
    // The repository's ignore rules keep execution run directories out of the clean-caller check.
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.gitignore"),
        root.join(".gitignore"),
    )
    .unwrap();
    f.commit();
    {
        // MA1, then MA2's acquisition and re-freeze, so MA4 has a fully frozen request.
        let _installed = f.install();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
        acquire(root, "0.2.0");
        f.commit();
        apply_prepare_plan(root, &build_prepare_plan(root, &args).unwrap()).unwrap();
        f.commit();
    }
    // The ordinary route keeps the production depth refusal.
    let refused = dry_run(root, &binary).unwrap_err();
    assert!(refused.contains("support_depth"), "{refused}");

    let _installed = f.install();
    let output = dry_run(root, &binary).unwrap();
    assert!(output.contains("depth_fixture"), "{output}");
    // Write mode reloads the request inside the independent candidate, which cannot resolve the
    // route itself, so it uses the captured registry. Gates are stubbed: they would run `cargo`.
    {
        let _gates = execute::gate_stub::stub();
        let written = run(root, &binary, true, "ma4-route").unwrap();
        assert!(written.contains("depth_fixture"), "{written}");
        // Only the checked candidate output reached the caller.
        let log = std::fs::read_to_string(root.join(LOG)).unwrap();
        assert!(log.ends_with("- executor note\n"), "{log}");
    }
    f.commit();

    // A host change while the executor runs refuses delivery: the caller is rechecked first.
    {
        let api = f.api.clone();
        let _gates = execute::gate_stub::stub_with(move || {
            api.put(
                "repos/fixture/isolated",
                json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
            )
        });
        let before = std::fs::read_to_string(root.join(LOG)).unwrap();
        let refused = run_as(root, &binary, "ma4-route-moved").unwrap_err();
        assert!(
            refused.contains("isolated maintenance route changed"),
            "{refused}"
        );
        assert_eq!(std::fs::read_to_string(root.join(LOG)).unwrap(), before);
    }

    // A failing installed route errors instead of falling back to the ordinary registry.
    f.api.put(
        "repos/fixture/isolated",
        json!({"id":11,"owner":{"id":20},"full_name":"fixture/isolated"}),
    );
    let failed = dry_run(root, &binary).unwrap_err();
    assert!(failed.contains("isolated maintenance route"), "{failed}");
}

#[test]
fn a_captured_registry_binds_only_an_independent_checkout_of_its_caller() {
    use crate::isolated_maintenance::{registry, RegistryError};
    let (f, _) = workspace();
    let root = f.root.path();
    let captured = {
        let _installed = f.install();
        crate::isolated_maintenance::route(root)
            .unwrap()
            .unwrap()
            .capture(root)
            .unwrap()
    };
    let scratch = tempfile::tempdir().unwrap();
    let candidate = scratch.path().join("candidate");
    let status = std::process::Command::new("git")
        .args(["clone", "--quiet", "--no-hardlinks"])
        .arg(root)
        .arg(&candidate)
        .status()
        .unwrap();
    assert!(status.success());
    // Neither the caller itself nor an unrelated directory can be bound.
    let unrelated = tempfile::tempdir().unwrap();
    assert!(captured.bind_candidate(root).is_err());
    assert!(captured.bind_candidate(unrelated.path()).is_err());
    {
        let _bound = captured.bind_candidate(&candidate).unwrap();
        assert!(registry(&candidate)
            .unwrap()
            .find("depth_fixture")
            .is_some());
        // Any other root keeps the ordinary load, and one thread binds one candidate at a time.
        assert!(matches!(
            registry(unrelated.path()),
            Err(RegistryError::Production(_))
        ));
        assert!(captured.bind_candidate(&candidate).is_err());
    }
    assert!(matches!(
        registry(&candidate),
        Err(RegistryError::Production(_))
    ));
}
