"""Offline, deterministic acquisition for the fixed Codex 0.156.1 pilot.

The supplied binary and upstream config schema must match their pinned digests.
This generates schemas only; it neither authenticates nor starts a model session.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

BINARY_SHA256 = "0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f"
CONFIG_SHA256 = "b373250e2565043b270d22df0e8cf32cacb36dd32fa2dc1e1e871115e73caab8"
COMMIT = "b412ff32c417f855c2b2d1581b77058eed87c84b"
CLIENT = ["initialize", "config/read", "configRequirements/read", "thread/start",
          "thread/resume", "thread/list", "thread/fork", "turn/start", "turn/interrupt"]
SERVER = ["item/commandExecution/requestApproval", "item/fileChange/requestApproval",
          "item/permissions/requestApproval", "item/tool/requestUserInput",
          "mcpServer/elicitation/request"]
NOTIFICATIONS = ["thread/started", "thread/closed", "turn/started", "turn/completed",
                 "item/started", "item/completed", "item/fileChange/patchUpdated",
                 "item/agentMessage/delta", "item/reasoning/textDelta", "serverRequest/resolved", "error"]
RESPONSES = ["v1/InitializeResponse", "v2/ConfigReadResponse", "v2/ConfigRequirementsReadResponse",
             "v2/ThreadStartResponse", "v2/ThreadResumeResponse", "v2/ThreadListResponse",
             "v2/ThreadForkResponse", "v2/TurnStartResponse", "v2/TurnInterruptResponse",
             "CommandExecutionRequestApprovalResponse", "FileChangeRequestApprovalResponse",
             "PermissionsRequestApprovalResponse", "ToolRequestUserInputResponse",
             "McpServerElicitationRequestResponse", "JSONRPCMessage", "ClientNotification"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def select_methods(schema, methods):
    selected = []
    for method in methods:
        matches = [row for row in schema["oneOf"]
                   if row.get("properties", {}).get("method", {}).get("enum") == [method]]
        if len(matches) != 1:
            raise ValueError(f"missing or ambiguous pinned method: {method}")
        selected.extend(matches)
    definitions = {}

    def visit(value):
        if isinstance(value, dict):
            for key, child in value.items():
                if key == "$ref":
                    prefix = "#/definitions/"
                    if not child.startswith(prefix) or "/" in child[len(prefix):]:
                        raise ValueError("unsupported schema reference")
                    name = child[len(prefix):]
                    if name not in definitions:
                        try:
                            definitions[name] = schema["definitions"][name]
                        except KeyError as error:
                            raise ValueError(f"missing definition: {name}") from error
                        visit(definitions[name])
                else:
                    visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)

    visit(selected)
    return {"$schema": schema.get("$schema"), "oneOf": selected, "definitions": definitions}


def build(schema_dir, config_bytes):
    schemas, raw = {}, {}
    for filename, methods in [("ClientRequest", CLIENT), ("ServerRequest", SERVER),
                              ("ServerNotification", NOTIFICATIONS)]:
        data = (schema_dir / (filename + ".json")).read_bytes()
        raw[filename + ".json"] = digest(data)
        schemas[filename] = select_methods(json.loads(data), methods)
    for filename in RESPONSES:
        data = (schema_dir / (filename + ".json")).read_bytes()
        raw[filename + ".json"] = digest(data)
        schemas[filename] = json.loads(data)
    return {
        "schema_version": 1, "version": "0.156.1", "target": "x86_64-unknown-linux-musl",
        "binary_sha256": BINARY_SHA256, "upstream_commit": COMMIT,
        "generator": ["app-server", "generate-json-schema", "--experimental", "--out", "<private>"],
        "source_schema_sha256": raw, "schemas": schemas,
        "config_source": f"https://raw.githubusercontent.com/openai/codex/{COMMIT}/codex-rs/core/config.schema.json",
        "config_schema_sha256": CONFIG_SHA256, "config_schema": json.loads(config_bytes),
        "shutdown": "transport stdin EOF followed by owned process cleanup; no shutdown RPC",
        "limits": "Schema/source correspondence only; no runtime defaults, execution, A1 or adequacy claim",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--config-schema", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    config = args.config_schema.read_bytes()
    if digest(binary.read_bytes()) != BINARY_SHA256 or digest(config) != CONFIG_SHA256:
        parser.error("binary or config schema does not match the pinned digest")
    with tempfile.TemporaryDirectory(prefix="codex-pilot-schema-") as temp:
        private = Path(temp)
        home = private / "home"
        home.mkdir()
        env = {"PATH": os.defpath, "HOME": str(home), "CODEX_HOME": str(home),
               "TMPDIR": temp, "LANG": "C.UTF-8", "TZ": "UTC"}
        subprocess.run([str(binary), "app-server", "generate-json-schema", "--experimental",
                        "--out", str(private / "schema")], cwd=private, env=env,
                       check=True, timeout=60, stdout=subprocess.DEVNULL)
        report = build(private / "schema", config)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    print(digest(args.output.read_bytes()))


if __name__ == "__main__":
    main()
