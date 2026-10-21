#!/usr/bin/env python3
"""Compare the committed staging ruleset with GitHub configuration without changing it."""

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


DESIRED = Path(__file__).resolve().parents[1] / "docs/agents/lifecycle/staging-integration-ruleset.json"
LEGACY_QUERY = """query($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    nameWithOwner viewerPermission
    ref(qualifiedName: "refs/heads/staging") { name branchProtectionRule { id } }
  }
}"""
FIELDS = {"name", "target", "enforcement", "bypass_actors", "conditions", "rules"}
METADATA = {"id", "node_id", "source_type", "source", "_links", "created_at", "updated_at"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def api(route, *, pages=False, fields=()):
    # Fixed REST GETs and one fixed GraphQL query; there is no mutation operation.
    argv = ["gh", "api", "--hostname", "github.com", "--method",
            "POST" if route == "graphql" else "GET", route,
            "-H", "Accept: application/vnd.github+json",
            "-H", "X-GitHub-Api-Version: 2026-03-10"]
    if pages:
        argv += ["--paginate", "--slurp"]
    for field in fields:
        argv += ["-f", field]
    result = subprocess.run(argv, capture_output=True, text=True, timeout=60)
    # Do not echo command stderr, which can contain credential/helper diagnostics.
    require(result.returncode == 0, f"GitHub read failed: {route.split('?')[0]}; check gh authentication/access")
    value = json.loads(result.stdout)
    if pages:
        require(isinstance(value, list) and value and all(isinstance(p, list) for p in value),
                f"invalid paginated readback: {route.split('?')[0]}")
        value = [item for page in value for item in page]
    return value


def canonical(value):
    """Current policy arrays are unordered; retain duplicates and all parameter fields."""
    if isinstance(value, dict):
        value = {key: canonical(item) for key, item in value.items()}
        if value.get("type") in ("deletion", "non_fast_forward") and value.get("parameters") == {}:
            del value["parameters"]
    elif isinstance(value, list):
        value = sorted((canonical(item) for item in value), key=lambda v: json.dumps(v, sort_keys=True))
    return value


def same(left, right):
    # JSON comparison preserves the distinction between false and 0.
    return json.dumps(canonical(left), sort_keys=True) == json.dumps(canonical(right), sort_keys=True)


def payload(value):
    require(isinstance(value, dict) and FIELDS <= value.keys(),
            "incomplete ruleset readback (including bypass_actors); verification unavailable")
    require(not value.keys() - FIELDS - METADATA, "unsupported ruleset fields in readback")
    return {key: value[key] for key in FIELDS}


def compare(repo, desired):
    require(isinstance(repo, str) and re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo)
            and all(part not in (".", "..") for part in repo.split("/")), "expected OWNER/REPO")
    require(isinstance(desired, dict) and set(desired) == FIELDS, "invalid desired ruleset fields")
    require(desired["target"] == "branch" and desired["enforcement"] == "active"
            and desired["conditions"] == {"ref_name": {"include": ["refs/heads/staging"], "exclude": []}},
            "desired ruleset must be active and target exactly refs/heads/staging")
    require(isinstance(desired["name"], str) and desired["name"]
            and isinstance(desired["rules"], list) and desired["rules"]
            and isinstance(desired["bypass_actors"], list), "invalid desired ruleset")
    prefix = f"repos/{repo}"
    listed = api(f"{prefix}/rulesets?includes_parents=true&per_page=100", pages=True)
    inventory = {}
    for item in listed:
        require(isinstance(item, dict) and type(item.get("id")) is int and item["id"] > 0
                and item["id"] not in inventory, "invalid or duplicate ruleset inventory ID")
        require(item.get("target") in ("branch", "tag", "push")
                and item.get("enforcement") in ("active", "disabled", "evaluate")
                and item.get("source_type") in ("Repository", "Organization", "Enterprise")
                and isinstance(item.get("source"), str) and item["source"]
                and isinstance(item.get("name"), str) and item["name"], "unsupported ruleset inventory")
        require(not (item["target"] == "push" and item["enforcement"] == "active"),
                "active push ruleset requires separate review; configuration verification unavailable")
        inventory[item["id"]] = item
    effective = api(f"{prefix}/rules/branches/staging?per_page=100", pages=True)
    for rule in effective:
        require(isinstance(rule, dict) and type(rule.get("ruleset_id")) is int
                and rule["ruleset_id"] in inventory and isinstance(rule.get("type"), str),
                "effective rule missing from readable inventory")
        item = inventory[rule["ruleset_id"]]
        require(rule.get("ruleset_source_type") == item["source_type"]
                and rule.get("ruleset_source") == item["source"] and item["enforcement"] == "active",
                "inconsistent effective rules/inventory; read configuration again")
    owner, name = repo.split("/")
    legacy = api("graphql", fields=(f"query={LEGACY_QUERY}", f"owner={owner}", f"name={name}"))
    require(isinstance(legacy, dict) and not legacy.get("errors"), "legacy protection query failed")
    repository = legacy.get("data", {}).get("repository")
    require(isinstance(repository, dict) and repository.get("nameWithOwner", "").casefold() == repo.casefold(),
            "legacy protection repository unavailable")
    require(repository.get("viewerPermission") == "ADMIN",
            "administrative readback visibility required for legacy protection")
    ref = repository.get("ref")
    require(isinstance(ref, dict) and ref.get("name") == "staging" and "branchProtectionRule" in ref,
            "staging ref/legacy protection unavailable")
    protection = ref["branchProtectionRule"]
    require(protection is None or (isinstance(protection, dict) and isinstance(protection.get("id"), str)
                                  and protection["id"]), "unsupported legacy protection readback")
    matches = [item for item in listed if item["name"] == desired["name"]
               and item["source_type"] == "Repository" and item["source"].casefold() == repo.casefold()]
    require(len(matches) <= 1, "ambiguous desired ruleset name")
    differences = []
    observed = None
    if not matches:
        differences.append("desired repository ruleset is not installed")
    else:
        item = matches[0]
        detail = api(f"{prefix}/rulesets/{item['id']}?includes_parents=true")
        require(isinstance(detail, dict) and all(detail.get(key) == item[key] for key in
                ("id", "name", "target", "source_type", "source", "enforcement")),
                "ruleset detail/inventory disagree; read configuration again")
        observed = payload(detail)
        differences += [f"desired ruleset differs: {key}" for key in sorted(FIELDS)
                        if not same(observed[key], desired[key])]
        rules = []
        for rule in effective:
            if rule["ruleset_id"] != item["id"]:
                differences.append(f"additional effective ruleset: {rule['ruleset_id']}")
            rules.append({key: value for key, value in rule.items()
                          if key not in ("ruleset_id", "ruleset_source", "ruleset_source_type")})
        if not same(rules, desired["rules"]):
            differences.append("effective staging rules differ from desired rules")
    if protection is not None:
        differences.append("overlapping legacy branch protection requires review")
    return {"status": "different" if differences else "match", "differences": sorted(set(differences)),
            "ruleset_id": matches[0]["id"] if matches else None, "observed_ruleset": observed,
            "effective_rules": effective, "ruleset_inventory": listed, "legacy_protection": protection}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="GitHub OWNER/REPO to read")
    args = parser.parse_args(argv)
    report = {"repository": args.repo, "branch": "refs/heads/staging",
              "observed_at": datetime.now(timezone.utc).isoformat(), "scope": "configuration comparison only"}
    try:
        content = DESIRED.read_bytes()
        report["desired_sha256"] = hashlib.sha256(content).hexdigest()
        report.update(compare(args.repo, json.loads(content)))
    except (OSError, ValueError, TypeError, KeyError, AttributeError, subprocess.TimeoutExpired) as error:
        # Detailed ValueError messages are safe; other failures must not echo credentials/data.
        report.update(status="unavailable", error=str(error) if isinstance(error, ValueError)
                      and not isinstance(error, json.JSONDecodeError) else "readback unavailable or malformed")
    print(json.dumps(report, indent=2, sort_keys=True))
    return {"match": 0, "different": 1, "unavailable": 2}[report["status"]]


if __name__ == "__main__":
    sys.exit(main())
