#!/usr/bin/env python3
"""Run the workspace test leg on a committed, credential-free declared fixture checkout."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


def run(arguments, root, **kwargs):
    return subprocess.check_output(arguments, cwd=root, text=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", default="target/s10-declared-test-leg")
    parser.add_argument("--lifecycle-path", choices=["onboarding", "maintenance"], default="onboarding")
    parser.add_argument("cargo_arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1]
    root = (source / args.destination).resolve()
    root.relative_to(source)
    root.mkdir(parents=True, exist_ok=False)
    files = run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], source)
    for relative in files.split("\0"):
        if not relative:
            continue
        original = source / relative
        if not original.exists():
            continue
        destination = root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        if original.is_symlink():
            destination.symlink_to(os.readlink(original))
        else:
            shutil.copy2(original, destination)

    fixtures = root / "crates/xtask/tests/fixtures/depth-synthetic"
    entry = (fixtures / "onboarding/registry-entry-deferred.toml" if args.lifecycle_path == "onboarding"
             else fixtures / "governing/registry-entry.toml").read_text()
    entry = entry.replace("{{ISOLATED_OWNER}}", "fixture").replace("{{ISOLATED_REPO}}", "isolated")
    registry = root / "crates/xtask/data/agent_registry.toml"
    registry.write_text(registry.read_text() + "\n" + entry)
    shutil.copytree(fixtures / "governing/manifest-root", root / "cli_manifests/depth_fixture")
    manifest = root / "cli_manifests/depth_fixture"
    (manifest / "current.json").write_text(json.dumps({"expected_targets": ["x86_64-unknown-linux-gnu"], "inputs": []}) + "\n")
    (manifest / "versions").mkdir()
    (manifest / "versions/.gitkeep").touch()
    governance = root / "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance"
    governance.mkdir(parents=True)
    approval = (fixtures / "onboarding/governance/approved-agent.toml").read_text()
    approval = approval.replace("{{APPROVAL_COMMIT}}", "abc1234").replace(
        "{{APPROVAL_RECORDED_AT}}", "2026-11-05T00:00:00Z")
    (governance / "approved-agent.toml").write_text(approval)
    state = json.loads((root / "docs/agents/lifecycle/aider-onboarding/governance/lifecycle-state.json")
                       .read_text().replace("aider-onboarding", "depth-fixture-cli-onboarding"))
    state["agent_id"] = "depth_fixture"
    state["onboarding_pack_prefix"] = "depth-fixture-cli-onboarding"
    state["approval_artifact_sha256"] = hashlib.sha256(approval.encode()).hexdigest()
    state["support_depth"] = {"policy_identity": "0" * 64}
    (governance / "lifecycle-state.json").write_text(json.dumps(state, indent=2) + "\n")
    (root / "producer").mkdir()
    (root / "producer/source.rs").write_text("approved producer\n")
    run(["git", "init", "-b", "staging"], root)
    for key, value in [("user.name", "Declared Test Fixture"), ("user.email", "fixture@example.invalid"),
                       ("commit.gpgsign", "false"), ("core.hooksPath", "/dev/null"),
                       ("maintenance.auto", "false"), ("gc.auto", "0")]:
        run(["git", "config", key, value], root)
    run(["git", "remote", "add", "origin", "https://github.com/fixture/isolated.git"], root)
    run(["git", "add", "."], root)
    run(["git", "commit", "-qm", "Committed depth-declared test fixture"], root)
    environment = os.environ.copy()
    environment.pop("TMPDIR", None)
    environment["CARGO_NET_OFFLINE"] = "true"
    environment["CARGO_TARGET_DIR"] = str(source / "target/s10-declared-build")
    # Cargo's shared-target dep-info can still name an earlier fixture's absolute source paths.
    # Rebuild workspace packages in this checkout while retaining cached external dependencies.
    metadata = json.loads(run(["cargo", "metadata", "--offline", "--no-deps", "--format-version=1"],
                              root, env=environment))
    members = set(metadata["workspace_members"])
    packages = [package["name"] for package in metadata["packages"] if package["id"] in members]
    clean = ["cargo", "clean", "--offline"]
    for package in packages:
        clean.extend(["-p", package])
    subprocess.run(clean, cwd=root, env=environment, check=True)
    build = run(["cargo", "test", "--offline", "-p", "xtask", "--lib", "--no-run",
                 "--message-format=json"], root, env=environment)
    artifacts = [json.loads(line) for line in build.splitlines() if line.startswith("{")]
    reader = next(item["executable"] for item in artifacts
                  if item.get("reason") == "compiler-artifact" and item.get("executable")
                  and item["target"]["kind"] == ["lib"])
    environment["S10_DECLARED_REGISTRY_READER"] = reader
    arguments = args.cargo_arguments
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    subprocess.run(["cargo", "test", "--offline", *(arguments or ["--workspace"])],
                   cwd=root, env=environment, check=True)
    assert not run(["git", "status", "--porcelain=v1", "--untracked-files=all"], root).strip()
    print(f"Declared {args.lifecycle_path} test leg passed in {root.relative_to(source)}")


if __name__ == "__main__":
    main()
