"""Offline native configuration observation, not packet evidence or qualification.

Linux/bubblewrap only. The child has an empty root, private writable roots, no
credentials and no external network. Only initialization and config reads are sent.
"""
import argparse
import json
import os
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time

from acquire_codex_pilot_protocol import BINARY_SHA256, digest

PROTOCOL = "cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json"
PROTOCOL_SHA256 = "964601de43f695ef07f1d992c4c14a7418346bdd8dbe90c4ac9123ec0a5e1fb2"
LIMIT = 2 * 1024 * 1024


def accepts_false(node, definitions):
    if "$ref" in node:
        prefix = "#/definitions/"
        if not node["$ref"].startswith(prefix):
            raise ValueError("unsupported schema reference")
        return accepts_false(definitions[node["$ref"][len(prefix):]], definitions)
    if "anyOf" in node:
        return any(accepts_false(child, definitions) for child in node["anyOf"])
    if node.get("type") == "boolean":
        return True
    if node.get("type") == "object":
        return False
    raise ValueError("unexpected pinned feature schema")


def expected_config(report):
    schema = report["config_schema"]
    config = ('model_provider = "openai"\nmodel = "gpt-5"\napproval_policy = "never"\n'
              'sandbox_mode = "workspace-write"\nweb_search = "disabled"\n'
              '[analytics]\nenabled = false\n[features]\n')
    unbound = []
    for key, node in sorted(schema["properties"]["features"]["properties"].items()):
        if accepts_false(node, schema["definitions"]):
            config += f"{key} = false\n"
        else:
            unbound.append(key)
    return config, unbound


def parse_features(text):
    rows = {}
    for line in text.splitlines():
        fields = line.split()
        if len(fields) < 3 or fields[-1] not in ("true", "false") or fields[0] in rows:
            raise ValueError("invalid or duplicate native feature row")
        rows[fields[0]] = {"stage": " ".join(fields[1:-1]), "enabled": fields[-1] == "true"}
    if not rows:
        raise ValueError("empty native feature listing")
    return rows


def manifest(root):
    return [{"path": str(p.relative_to(root)), "sha256": digest(p.read_bytes())}
            for p in sorted(root.rglob("*")) if p.is_file()]


def namespace_command(bwrap, binary, private):
    # Empty root prevents ancestor/system/project loader inputs leaking from the host.
    command = [bwrap, "--unshare-all", "--die-with-parent", "--new-session", "--hostname",
               "codex-pilot", "--tmpfs", "/", "--proc", "/proc", "--dev", "/dev",
               "--dir", "/bin", "--dir", "/etc", "--dir", "/tmp",
               "--bind", str(private), "/pilot", "--ro-bind", str(binary), "/bin/codex",
               "--chdir", "/pilot/work", "--clearenv"]
    environment = {"PATH": "/bin", "HOME": "/pilot/home", "CODEX_HOME": "/pilot/home",
                   "CODEX_BINARY": "/bin/codex", "RUST_LOG": "error", "TMPDIR": "/pilot/tmp",
                   "XDG_CONFIG_HOME": "/pilot/xdg/config", "XDG_DATA_HOME": "/pilot/xdg/data",
                   "XDG_CACHE_HOME": "/pilot/xdg/cache", "LANG": "C.UTF-8", "TZ": "UTC"}
    for key, value in environment.items():
        command += ["--setenv", key, value]
    return command + ["/bin/codex"], environment


def config_reads(command, stderr):
    transcript, buffer = [], b""
    process = subprocess.Popen(command + ["app-server"], stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=stderr)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    methods = [("initialize", {"clientInfo": {"name": "codex_pilot_context", "version": "0.1"},
                               "capabilities": {"experimentalApi": True}}),
               ("config/read", {"cwd": "/pilot/work", "includeLayers": True}),
               ("configRequirements/read", {})]
    try:
        for identity, (method, params) in enumerate(methods, 1):
            request = {"id": identity, "method": method, "params": params}
            transcript.append({"sent": request})
            process.stdin.write((json.dumps(request) + "\n").encode())
            process.stdin.flush()
            deadline, received = time.monotonic() + 15, 0
            while True:
                if time.monotonic() >= deadline:
                    raise TimeoutError(method)
                if b"\n" not in buffer:
                    if not selector.select(max(0, deadline - time.monotonic())):
                        raise TimeoutError(method)
                    chunk = os.read(process.stdout.fileno(), 65536)
                    if not chunk:
                        raise ValueError("native server closed before reply")
                    received += len(chunk)
                    if received > LIMIT:
                        raise ValueError("native response exceeds observation limit")
                    buffer += chunk
                    continue
                line, buffer = buffer.split(b"\n", 1)
                response = json.loads(line)
                transcript.append({"received": response})
                if "method" in response and "id" in response:
                    raise ValueError("unexpected interactive native request")
                if response.get("id") == identity:
                    if "error" in response or "result" not in response:
                        raise ValueError(f"native {method} refused")
                    break
    finally:
        process.stdin.close()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise ValueError("native server required forced cleanup")
        finally:
            selector.close()
            process.stdout.close()
    if process.returncode:
        raise ValueError("native server exited unsuccessfully")
    return transcript


def observe(binary, protocol, bwrap):
    raw = protocol.read_bytes()
    if digest(raw) != PROTOCOL_SHA256 or digest(binary.read_bytes()) != BINARY_SHA256:
        raise ValueError("binary or protocol report differs from the pinned acquisition")
    config, unbound = expected_config(json.loads(raw))
    with tempfile.TemporaryDirectory(prefix="codex-context-") as temp:
        root = Path(temp)
        for name in ("home", "work", "tmp", "xdg/config", "xdg/data", "xdg/cache"):
            (root / name).mkdir(parents=True, exist_ok=True)
        (root / "home/config.toml").write_text(config)
        command, environment = namespace_command(bwrap, binary, root)
        before = manifest(root)
        with tempfile.TemporaryFile() as stderr:
            transcript = config_reads(command, stderr)
            stderr.seek(0)
            diagnostics = stderr.read(LIMIT + 1)
            if len(diagnostics) > LIMIT:
                raise ValueError("native diagnostics exceed observation limit")
        after_config_reads = manifest(root)
        features = subprocess.run(command + ["features", "list"], check=True, timeout=20,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if len(features.stdout) + len(features.stderr) > LIMIT:
            raise ValueError("native feature listing exceeds observation limit")
        return {"schema_version": 1, "version": "0.156.1", "target": "x86_64-unknown-linux-musl",
                "binary_sha256": BINARY_SHA256, "protocol_sha256": PROTOCOL_SHA256,
                "collector_sha256": digest(Path(__file__).read_bytes()),
                "acquisition_helper_sha256": digest(Path(__file__).with_name("acquire_codex_pilot_protocol.py").read_bytes()),
                "isolation": "bubblewrap: empty root, private pid/user/net namespaces; no credentials",
                "launcher_sha256": digest(Path(bwrap).read_bytes()),
                "environment": environment, "expected_config_toml": config,
                "unbound_object_feature_defaults": unbound,
                "before": before, "after_config_reads": after_config_reads, "after": manifest(root),
                "transcript": transcript, "features_list": parse_features(features.stdout.decode()),
                "diagnostics": diagnostics.decode(errors="replace"),
                "feature_diagnostics": features.stderr.decode(errors="replace"),
                "limits": ["Config-only direct native launch; not a production adapter spawn observation",
                           "No thread/turn, model, approval, command or file-change execution",
                           "Config projection and feature listing do not expose every resolved internal default",
                           "No packet E, A1 attestation, independent adequacy, enrollment or qualification"]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    bwrap = shutil.which("bwrap")
    if not bwrap:
        parser.error("bubblewrap is required; no unisolated fallback")
    report = observe(args.binary.resolve(strict=True), Path(__file__).resolve().parents[1] / PROTOCOL, bwrap)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(digest(args.output.read_bytes()))


if __name__ == "__main__":
    main()
