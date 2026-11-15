//! OR-P13 S/T leaves run in a disposable Git checkout without depth-host authority.
use super::*;

const COMMANDS: &[&str] = &[
    "make hygiene",
    "make fmt-check",
    "python3 -m unittest discover -s scripts -p 'test_*.py'",
    "make clippy",
    "cargo clean",
    "make check",
    "cargo test --workspace --all-targets --all-features",
    "make publish-guards",
    "make loc-check",
    "make actionlint",
    "make security",
    "make unsafe-report",
];

pub(super) fn run(
    caller: &Path,
    proposed: &Path,
    repository: &str,
    written: &[PathBuf],
) -> Result<(), String> {
    let commands: Vec<_> = COMMANDS.iter().map(|s| (*s).to_owned()).collect();
    #[cfg(test)]
    if let Some(runner) = TEST_GATE_RUNNER.with(|slot| *slot.borrow()) {
        return runner(proposed, &commands);
    }
    let initial = crate::candidate_execution::tree::capture(proposed, &Default::default())
        .map_err(|e| e.to_string())?;
    let checkout = tooling_checkout(caller, proposed, repository, written)?;
    let cache = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or("ON4 gates require an offline dependency cache")?;
    let output = Command::new("python3")
        .args(["-c", include_str!("gates.py")])
        .arg(checkout.path())
        .arg(cache)
        .arg(serde_json::to_string(&commands).map_err(|e| e.to_string())?)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "ON4 proposal subprocess gates failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if crate::candidate_execution::tree::capture(checkout.path(), &Default::default())
        .map_err(|e| e.to_string())?
        != initial
    {
        return Err("ON4 subprocess gates changed proposed inputs".into());
    }
    Ok(())
}

/// Preserve the caller's tracked-path inventory, including ignored tracked artifacts. Only
/// this ON4 batch adds paths. Other observed scratch files remain outside the tooling index.
fn tooling_checkout(
    caller: &Path,
    proposed: &Path,
    repository: &str,
    written: &[PathBuf],
) -> Result<tempfile::TempDir, String> {
    let checkout = tempfile::tempdir().map_err(|e| e.to_string())?;
    let initial = crate::candidate_execution::tree::capture(proposed, &Default::default())
        .map_err(|e| e.to_string())?;
    crate::candidate_execution::tree::materialize(checkout.path(), &initial)
        .map_err(|e| e.to_string())?;
    let remote = format!("https://github.com/{repository}.git");
    let git = |root: &Path| {
        let mut command = crate::git_inputs::command(root);
        command
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null");
        command
    };
    for args in [
        vec!["init", "--template=", "-b", "candidate"],
        vec!["config", "user.name", "ON4 local checks"],
        vec!["config", "user.email", "on4@example.invalid"],
        vec!["config", "commit.gpgsign", "false"],
        vec!["config", "core.hooksPath", "/dev/null"],
        vec!["config", "maintenance.auto", "false"],
        vec!["config", "gc.auto", "0"],
        vec!["remote", "add", "origin", &remote],
    ] {
        let output = git(checkout.path())
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "ON4 gate checkout failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    let tracked = crate::git_inputs::command(caller)
        .args(["ls-files", "--cached", "-z"])
        .output()
        .map_err(|e| e.to_string())?;
    if !tracked.status.success() {
        return Err("ON4 gates cannot read the captured caller index".into());
    }
    let mut paths: std::collections::BTreeSet<_> = std::str::from_utf8(&tracked.stdout)
        .map_err(|e| e.to_string())?
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect();
    for path in written {
        paths.insert(path.to_str().ok_or("non-UTF8 ON4 gate output")?.to_owned());
    }
    let mut regular = Vec::new();
    for path in paths {
        match initial.get(&path) {
            Some(crate::candidate_execution::tree::Node::File(..)) => regular.push(path),
            None => {}
            Some(_) => return Err("ON4 gate tracked path is not a regular file".into()),
        }
    }
    let mut child = git(checkout.path())
        .args([
            "--literal-pathspecs",
            "add",
            "--force",
            "--pathspec-from-file=-",
            "--pathspec-file-nul",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    {
        let mut input = child.stdin.take().ok_or("missing ON4 gate index input")?;
        for path in regular {
            input
                .write_all(path.as_bytes())
                .and_then(|_| input.write_all(&[0]))
                .map_err(|e| e.to_string())?;
        }
    }
    if !child
        .wait_with_output()
        .map_err(|e| e.to_string())?
        .status
        .success()
    {
        return Err("ON4 gates cannot stage the exact candidate paths".into());
    }
    let committed = git(checkout.path())
        .args(["commit", "-qm", "ON4 proposed tooling inputs"])
        .output()
        .map_err(|e| e.to_string())?;
    if !committed.status.success() {
        return Err(format!(
            "ON4 gate commit failed: {}",
            String::from_utf8_lossy(&committed.stderr)
        ));
    }
    Ok(checkout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_on4_tooling_checkout_preserves_ignored_tracked_and_owned_paths() {
        let caller = tempfile::tempdir().unwrap();
        let proposed = tempfile::tempdir().unwrap();
        for root in [caller.path(), proposed.path()] {
            fs::create_dir(root.join("docs")).unwrap();
            fs::write(root.join(".gitignore"), "docs/*.json\n").unwrap();
            fs::write(root.join("docs/prior.json"), "prior\n").unwrap();
        }
        for args in [
            vec!["init", "--template="],
            vec!["add", "--force", ".gitignore", "docs/prior.json"],
        ] {
            let output = crate::git_inputs::command(caller.path())
                .args(args)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        }
        fs::write(proposed.path().join("docs/publication.json"), "owned\n").unwrap();
        fs::write(proposed.path().join("docs/scratch.json"), "scratch\n").unwrap();
        let before =
            crate::candidate_execution::tree::capture(caller.path(), &Default::default()).unwrap();
        let checkout = tooling_checkout(
            caller.path(),
            proposed.path(),
            "fixture/isolated",
            &["docs/publication.json".into()],
        )
        .unwrap();
        let output = crate::git_inputs::command(checkout.path())
            .args(["ls-files"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            ".gitignore\ndocs/prior.json\ndocs/publication.json\n"
        );
        assert_eq!(
            fs::read_to_string(checkout.path().join("docs/scratch.json")).unwrap(),
            "scratch\n"
        );
        assert_eq!(
            crate::candidate_execution::tree::capture(caller.path(), &Default::default()).unwrap(),
            before
        );
    }

    #[test]
    fn public_on4_subprocess_inventory_uses_private_environments_and_independent_reader() {
        let root = tempfile::tempdir().unwrap();
        let scripts = root.path().join("scripts");
        fs::create_dir(&scripts).unwrap();
        fs::write(
            scripts.join("s10_declared_test_leg.py"),
            include_str!("../../../../../scripts/s10_declared_test_leg.py"),
        )
        .unwrap();
        let cache = tempfile::tempdir().unwrap();
        let commands: Vec<_> = COMMANDS.iter().map(|value| (*value).to_owned()).collect();
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../isolated_maintenance/tests/bootstrap/vectors/on4-publication.json"
        ))
        .unwrap();
        assert_eq!(
            serde_json::json!(commands),
            expected["proposal_subprocesses"]
        );
        let harness = r#"
import json, os, subprocess, sys
from pathlib import Path
source = sys.argv.pop()
expected = json.loads(sys.argv[3])
root = Path(sys.argv[1])
seen = []
def output(arguments, **kwargs):
    environment = kwargs['env']
    assert 'ON4_TEST_CREDENTIAL' not in environment
    assert 'GIT_CONFIG_COUNT' not in environment
    assert environment['CARGO_NET_OFFLINE'] == 'true'
    if arguments[:2] == ['rustup', 'show']:
        return 'stable active\n'
    if arguments[:2] == ['rustup', 'run']:
        return 'rustc installed\n'
    assert arguments[:3] == ['cargo', 'test', '--offline']
    return json.dumps({'reason':'compiler-artifact', 'executable':'independent-reader', 'target':{'kind':['lib']}})
def run(arguments, **kwargs):
    assert arguments[:2] == ['sh', '-c']
    assert kwargs['cwd'] == root
    assert kwargs['check']
    environment = kwargs['env']
    assert 'ON4_TEST_CREDENTIAL' not in environment
    assert 'GIT_CONFIG_COUNT' not in environment
    assert Path(environment['HOME']).is_relative_to(root / 'target')
    assert Path(environment['CARGO_HOME']).is_relative_to(root / 'target')
    assert environment['RUSTUP_AUTO_INSTALL'] == '0'
    command = arguments[2]
    if command == 'cargo test --workspace --all-targets --all-features':
        assert environment['S10_DECLARED_REGISTRY_READER'] == 'independent-reader'
    else:
        assert 'S10_DECLARED_REGISTRY_READER' not in environment
    seen.append(command)
subprocess.check_output = output
subprocess.run = run
exec(compile(source, 'ON4 S/T runner', 'exec'))
assert seen == expected
"#;
        let output = Command::new("python3")
            .args(["-c", harness])
            .arg(root.path())
            .arg(cache.path())
            .arg(serde_json::to_string(&commands).unwrap())
            .arg(include_str!("gates.py"))
            .env("ON4_TEST_CREDENTIAL", "test-only-sentinel")
            .env("GIT_CONFIG_COUNT", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
