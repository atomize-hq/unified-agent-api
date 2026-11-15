"""Run ON4's S/T inventory with private homes, offline caches and test-owned readers."""
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1])
cache = Path(sys.argv[2])
commands = json.loads(sys.argv[3])
sys.path.insert(0, str(root / "scripts"))
from s10_declared_test_leg import (copy_files, dependency_files, environment_for,
                                   require_installed_toolchain)

environment = environment_for(root)
Path(environment["HOME"]).mkdir(parents=True)
copy_files(cache, Path(environment["CARGO_HOME"]), dependency_files(cache))
require_installed_toolchain(root, environment)
# Security's Makefile recipe selects its own private Cargo home.
copy_files(cache, root / "target/security-cargo-home", dependency_files(cache))

for command in commands:
    if command == "cargo test --workspace --all-targets --all-features":
        # Each candidate-registry test initializes its own fake host in a child. No
        # controller route, verdict, reusable gate receipt or credential is inherited.
        build = subprocess.check_output([
            "cargo", "test", "--offline", "-p", "xtask", "--lib", "--no-run",
            "--message-format=json",
        ], cwd=root, env=environment, text=True)
        artifacts = [json.loads(line) for line in build.splitlines() if line.startswith("{")]
        reader = next(item["executable"] for item in artifacts
                      if item.get("reason") == "compiler-artifact" and item.get("executable")
                      and item["target"]["kind"] == ["lib"])
        test_environment = dict(environment, S10_DECLARED_REGISTRY_READER=reader)
        subprocess.run(["sh", "-c", command], cwd=root, env=test_environment, check=True)
    else:
        subprocess.run(["sh", "-c", command], cwd=root, env=environment, check=True)
