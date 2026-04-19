#!/usr/bin/env python3
"""Validate publish metadata and package/publish readiness for workspace crates."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from publish_planner import (
    WorkspacePackage,
    load_metadata,
    load_publishable_packages,
    package_map,
    run,
    topological_publish_order,
)

REQUIRED_LICENSE_EXPRESSION = "MIT OR Apache-2.0"

REQUIRED_METADATA_FIELDS = [
    "description",
    "repository",
    "homepage",
    "documentation",
    "readme",
    "license",
]

REQUIRED_PACKAGE_FILES = {
    "Cargo.toml",
    "Cargo.toml.orig",
    "README.md",
    "LICENSE-APACHE",
    "LICENSE-MIT",
}

BANNED_PACKAGE_PREFIXES = (
    ".github/",
    "_download/",
    "_extract/",
    "target/",
    "wt/",
)
def run_dependent_package_check(root: Path, package: WorkspacePackage) -> None:
    cmd = [
        "cargo",
        "package",
        "--allow-dirty",
        "--locked",
        "--no-verify",
        "-p",
        package.name,
    ]
    result = run(
        cmd,
        cwd=root,
        check=False,
        capture_output=True,
    )
    if result.returncode == 0:
        return

    stderr = result.stderr
    bootstrap_lookup_error = any(dep in stderr for dep in package.internal_dependencies) and (
        "no matching package named" in stderr
        or (
            "failed to select a version for the requirement" in stderr
            and "location searched: crates.io index" in stderr
        )
    )
    if bootstrap_lookup_error:
        print(
            "Skipping strict cargo package for "
            f"{package.name}: dependent crate is not yet visible in crates.io during bootstrap.",
            file=sys.stderr,
        )
        return

    result.check_returncode()


def package_path(package: dict, field: str) -> Path:
    manifest_path = Path(package["manifest_path"])
    return (manifest_path.parent / package[field]).resolve()


def validate_metadata(packages: list[WorkspacePackage], metadata: dict) -> list[str]:
    package_by_name = package_map(packages)
    package_metadata_by_name = {
        package["name"]: package
        for package in metadata["packages"]
        if package["name"] in package_by_name
    }
    errors: list[str] = []
    for name in package_by_name:
        package = package_metadata_by_name.get(name)
        if package is None:
            errors.append(f"workspace package {name} is missing from cargo metadata")
            continue

        for field in REQUIRED_METADATA_FIELDS:
            value = package.get(field)
            if not value:
                errors.append(f"{name} is missing required metadata field `{field}`")

        license_value = package.get("license")
        if license_value and license_value != REQUIRED_LICENSE_EXPRESSION:
            errors.append(
                f"{name} must set `license` to `{REQUIRED_LICENSE_EXPRESSION}`, found `{license_value}`"
            )

        if package.get("license_file"):
            errors.append(f"{name} must not set `license_file`; use SPDX `license` metadata only")

        for field in ("readme",):
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
    packages = load_publishable_packages(root)
    ordered_packages = topological_publish_order(packages)

    errors = validate_metadata(packages, metadata)
    for package in ordered_packages:
        errors.extend(validate_package_listing(root, package.name))

    if errors:
        print("Publish readiness validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    for package in ordered_packages:
        if package.has_internal_dependencies:
            run_dependent_package_check(root, package)
            continue

        run(
            [
                "cargo",
                "publish",
                "--dry-run",
                "--allow-dirty",
                "--locked",
                "-p",
                package.name,
            ],
            cwd=root,
        )

    print(
        "Publish readiness checks passed for "
        + ", ".join(package.name for package in ordered_packages)
        + "."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
