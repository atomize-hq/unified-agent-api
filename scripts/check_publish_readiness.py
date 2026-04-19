#!/usr/bin/env python3
"""Validate publish metadata and package/publish readiness for workspace crates."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

LEAF_PACKAGES = [
    "unified-agent-api-codex",
    "unified-agent-api-claude-code",
    "unified-agent-api-opencode",
]

DEPENDENT_PACKAGES = [
    "unified-agent-api-wrapper-events",
    "unified-agent-api",
]

ALL_PACKAGES = LEAF_PACKAGES + DEPENDENT_PACKAGES

REQUIRED_METADATA_FIELDS = [
    "description",
    "repository",
    "homepage",
    "documentation",
    "readme",
    "license_file",
]

REQUIRED_PACKAGE_FILES = {
    "Cargo.toml",
    "Cargo.toml.orig",
    "README.md",
    "LICENSE",
}

BANNED_PACKAGE_PREFIXES = (
    ".github/",
    "_download/",
    "_extract/",
    "target/",
    "wt/",
)


def run(cmd: list[str], *, cwd: Path, capture_output: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        check=True,
        capture_output=capture_output,
        text=True,
    )


def run_dependent_package_check(root: Path, package_name: str) -> None:
    cmd = [
        "cargo",
        "package",
        "--allow-dirty",
        "--locked",
        "--no-verify",
        "-p",
        package_name,
    ]
    result = subprocess.run(
        cmd,
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return

    stderr = result.stderr
    bootstrap_lookup_error = any(dep in stderr for dep in LEAF_PACKAGES) and (
        "no matching package named" in stderr
        or (
            "failed to select a version for the requirement" in stderr
            and "location searched: crates.io index" in stderr
        )
    )
    if bootstrap_lookup_error:
        print(
            f"Skipping strict cargo package for {package_name}: dependent crate is not yet visible in crates.io during bootstrap.",
            file=sys.stderr,
        )
        return

    raise subprocess.CalledProcessError(
        result.returncode,
        cmd,
        output=result.stdout,
        stderr=result.stderr,
    )


def load_metadata(root: Path) -> dict:
    result = run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=root,
        capture_output=True,
    )
    return json.loads(result.stdout)


def package_path(package: dict, field: str) -> Path:
    manifest_path = Path(package["manifest_path"])
    return (manifest_path.parent / package[field]).resolve()


def validate_metadata(root: Path, metadata: dict) -> list[str]:
    workspace_members = set(metadata["workspace_members"])
    package_by_name = {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_members
    }

    errors: list[str] = []
    for name in ALL_PACKAGES:
        package = package_by_name.get(name)
        if package is None:
            errors.append(f"workspace package {name} is missing from cargo metadata")
            continue

        for field in REQUIRED_METADATA_FIELDS:
            value = package.get(field)
            if not value:
                errors.append(f"{name} is missing required metadata field `{field}`")

        for field in ("readme", "license_file"):
            value = package.get(field)
            if not value:
                continue
            path = package_path(package, field)
            if not path.exists():
                errors.append(f"{name} points `{field}` at missing path {path}")

    return errors


def validate_package_listing(root: Path, package_name: str) -> list[str]:
    result = run(
        [
            "cargo",
            "package",
            "-p",
            package_name,
            "--list",
            "--allow-dirty",
            "--locked",
            "--no-verify",
        ],
        cwd=root,
        capture_output=True,
    )
    entries = {line.strip() for line in result.stdout.splitlines() if line.strip()}

    errors: list[str] = []
    for required in sorted(REQUIRED_PACKAGE_FILES):
        if required not in entries:
            errors.append(f"{package_name} package listing is missing required file `{required}`")

    for entry in sorted(entries):
        if any(entry == prefix.rstrip("/") or entry.startswith(prefix) for prefix in BANNED_PACKAGE_PREFIXES):
            errors.append(f"{package_name} package listing contains banned path `{entry}`")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".", help="workspace root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    metadata = load_metadata(root)

    errors = validate_metadata(root, metadata)
    for package_name in ALL_PACKAGES:
        errors.extend(validate_package_listing(root, package_name))

    if errors:
        print("Publish readiness validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    for package_name in LEAF_PACKAGES:
        run(
            [
                "cargo",
                "publish",
                "--dry-run",
                "--allow-dirty",
                "--locked",
                "-p",
                package_name,
            ],
            cwd=root,
        )

    for package_name in DEPENDENT_PACKAGES:
        run_dependent_package_check(root, package_name)

    print(
        "Publish readiness checks passed for "
        + ", ".join(ALL_PACKAGES)
        + "."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
