#!/usr/bin/env python3
"""Validate publishable workspace package versions and inter-crate pins."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

from publish_planner import is_crates_io_publishable


def load_root_version(root: Path) -> str:
    version = (root / "VERSION").read_text(encoding="utf-8").strip()
    if not version:
        raise SystemExit("VERSION is empty.")
    return version


def load_metadata(root: Path) -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".", help="workspace root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    root_version = load_root_version(root)
    metadata = load_metadata(root)

    workspace_member_ids = set(metadata["workspace_members"])
    workspace_packages = [
        package for package in metadata["packages"] if package["id"] in workspace_member_ids
    ]
    publishable_packages = [
        package for package in workspace_packages if is_crates_io_publishable(package)
    ]
    publishable_names = {package["name"] for package in publishable_packages}

    errors: list[str] = []
    expected_dep_req = f"={root_version}"

    for package in sorted(publishable_packages, key=lambda item: item["name"]):
        if package["version"] != root_version:
            errors.append(
                f"{package['name']} resolves to version {package['version']}, expected {root_version}."
            )

        for dep in sorted(package["dependencies"], key=lambda item: item["name"]):
            if dep["name"] not in publishable_names:
                continue
            if dep.get("source") is not None:
                continue

            req = dep.get("req")
            if not req:
                errors.append(
                    f"{package['name']} -> {dep['name']} is missing an explicit version requirement."
                )
                continue
            if req != expected_dep_req:
                errors.append(
                    f"{package['name']} -> {dep['name']} uses {req}; expected {expected_dep_req}."
                )

    if errors:
        print("Crates.io-publishable workspace version validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(
        "Validated "
        f"{len(publishable_packages)} crates.io-publishable workspace packages "
        f"against VERSION={root_version}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
