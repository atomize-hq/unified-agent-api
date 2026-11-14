#!/usr/bin/env python3
"""Run the workspace test leg on a committed, credential-free declared fixture checkout.

The runner does not mutate the caller's repository, credentials or Git metadata, and
writes only inside its destination. It uses the same toolchain/config trust as the
ordinary test leg and make preflight: Cargo's ancestor .cargo/config.toml search
remains active. Cargo configuration is not hermetic. Toolchains must already be
installed; the runner disables rustup auto-installation and never installs them.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess


def run(arguments, root, environment):
    return subprocess.check_output(arguments, cwd=root, text=True, env=environment)


def environment_for(root):
    """Only tool discovery and Rust toolchain selection come from the caller."""
    state = root / "target/s10-runner"
    environment = {"PATH": os.environ.get("PATH", os.defpath),
                   "HOME": str(state / "home"),
                   "XDG_CONFIG_HOME": str(state / "home/.config"),
                   "CARGO_HOME": str(state / "cargo-home"),
                   "CARGO_TARGET_DIR": str(state / "build"),
                   "CARGO_NET_OFFLINE": "true",
                   "RUSTUP_AUTO_INSTALL": "0",
                   "RUSTUP_HOME": str(Path(os.environ.get("RUSTUP_HOME", Path.home() / ".rustup")).resolve())}
    if "RUSTUP_TOOLCHAIN" in os.environ:
        environment["RUSTUP_TOOLCHAIN"] = os.environ["RUSTUP_TOOLCHAIN"]
    return environment


def test_selection_arguments(arguments):
    """Allow only test selection; keep Cargo's containment controls runner-owned."""
    arguments = arguments[1:] if arguments[:1] == ["--"] else arguments
    libtest = False
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        flags = ({"--exact", "--nocapture", "--ignored", "--include-ignored"} if libtest
                 else {"--lib", "--workspace", "--all-targets", "--all-features"})
        if argument == "--" and not libtest:
            libtest = True
        elif argument in flags or not argument.startswith("-"):
            pass  # Bare strings are test-name filters.
        else:
            option, separator, value = argument.partition("=")
            if not libtest and argument.startswith("-p") and argument != "-p":
                option, separator, value = "-p", "=", argument[2:]
            value_options = {"--test-threads"} if libtest else {"-p", "--package", "--test"}
            if option not in value_options:
                kind = "libtest" if libtest else "Cargo"
                raise ValueError(f"unsupported {kind} test-selection argument: {argument!r}")
            if not separator:
                index += 1
                value = arguments[index] if index < len(arguments) else ""
            if not value or value.startswith("-"):
                raise ValueError(f"{option} requires a non-option value")
            if option == "--test-threads" and (
                    not value.isascii() or not value.isdecimal() or int(value) < 1):
                raise ValueError("--test-threads requires a positive integer")
        index += 1
    return arguments


def require_installed_toolchain(root, environment):
    toolchain = environment.get("RUSTUP_TOOLCHAIN")
    try:
        if not toolchain:
            toolchain = run(["rustup", "show", "active-toolchain"], root, environment).split()[0]
        if toolchain.startswith("-"):
            raise ValueError("Rust toolchain selection must be a name or path, not an option")
        run(["rustup", "run", toolchain, "rustc", "--version"], root, environment)
    except (OSError, subprocess.CalledProcessError, IndexError) as error:
        raise ValueError(f"selected Rust toolchain {toolchain or '(active)'!r} must be already "
                         "installed and runnable; the runner never installs toolchains") from error
    environment["RUSTUP_TOOLCHAIN"] = toolchain


def regular_files(source, relatives):
    """Validate the entire copy plan before writing anything; never preserve symlinks."""
    files = []
    for relative in relatives:
        if not relative:
            continue
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError(f"copy path escapes source: {relative}")
        original = source
        for part in path.parts:
            original = original / part
            if original.is_symlink():
                raise ValueError(f"symlink is not a regular copy input: {relative}")
        if not original.exists():  # Tracked deletions are absent from this snapshot.
            continue
        if not stat.S_ISREG(original.stat().st_mode):
            raise ValueError(f"non-regular copy input: {relative}")
        files.append(path)
    return files


def dependency_files(cache):
    # Snapshot only offline registry data. Home config, credentials and executables are excluded.
    # The input is only read; Cargo can write solely to its private copy, never this cache.
    relatives = []
    for relative in ["registry/index", "registry/cache"]:
        directory = cache / relative
        if directory.is_symlink():
            raise ValueError(f"symlink is not a dependency-cache input: {relative}")
        if directory.exists():
            for parent, directories, files in os.walk(directory):
                for name in directories + files:
                    path = Path(parent) / name
                    if path.is_symlink():
                        raise ValueError(f"symlink is not a dependency-cache input: {path}")
                relatives.extend(str((Path(parent) / name).relative_to(cache)) for name in files)
    return regular_files(cache, relatives)


def copy_files(source, destination, files):
    for relative in files:
        output = destination / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / relative, output)
        output.chmod(stat.S_IMODE((source / relative).stat().st_mode) | stat.S_IWUSR)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--destination", default="target/s10-declared-test-leg")
    parser.add_argument("--lifecycle-path", choices=["onboarding", "maintenance"], default="onboarding")
    parser.add_argument("--dependency-cache", type=Path,
                        default=Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")),
                        help="read-only input: copy only registry index and crate archives")
    parser.add_argument("cargo_arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        arguments = test_selection_arguments(args.cargo_arguments)
    except ValueError as error:
        parser.error(str(error))
    source = Path(__file__).resolve().parents[1]
    root = (source / args.destination).resolve()
    root.relative_to(source)
    environment = environment_for(root)
    files = run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], source,
                environment)
    files = regular_files(source, files.split("\0"))
    cache = args.dependency_cache.resolve()
    dependencies = dependency_files(cache)
    root.mkdir(parents=True, exist_ok=False)
    copy_files(source, root, files)
    Path(environment["HOME"]).mkdir(parents=True)
    copy_files(cache, Path(environment["CARGO_HOME"]), dependencies)
    try:
        require_installed_toolchain(root, environment)
    except ValueError as error:
        parser.error(str(error))

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
    run(["git", "init", "--template=", "-b", "staging"], root, environment)
    (root / ".git/info").mkdir(exist_ok=True)
    (root / ".git/info/exclude").write_text("/target/\n")
    for key, value in [("user.name", "Declared Test Fixture"), ("user.email", "fixture@example.invalid"),
                       ("commit.gpgsign", "false"), ("core.hooksPath", "/dev/null"),
                       ("maintenance.auto", "false"), ("gc.auto", "0")]:
        run(["git", "config", key, value], root, environment)
    run(["git", "remote", "add", "origin", "https://github.com/fixture/isolated.git"], root, environment)
    run(["git", "add", "."], root, environment)
    run(["git", "commit", "-qm", "Committed depth-declared test fixture"], root, environment)
    build = run(["cargo", "test", "--offline", "-p", "xtask", "--lib", "--no-run",
                 "--message-format=json"], root, environment)
    artifacts = [json.loads(line) for line in build.splitlines() if line.startswith("{")]
    reader = next(item["executable"] for item in artifacts
                  if item.get("reason") == "compiler-artifact" and item.get("executable")
                  and item["target"]["kind"] == ["lib"])
    environment["S10_DECLARED_REGISTRY_READER"] = reader
    subprocess.run(["cargo", "test", "--offline", *(arguments or ["--workspace", "--all-targets", "--all-features"])],
                   cwd=root, env=environment, check=True)
    assert not run(["git", "status", "--porcelain=v1", "--untracked-files=all"], root, environment).strip()
    print(f"Declared {args.lifecycle_path} test leg passed in {root.relative_to(source)}")


if __name__ == "__main__":
    main()
