"""Observe real SDK child spawns offline. Produces an audit, never P/O/E or qualification.

Requires a clean committed checkout, a local Rust toolchain, Linux/bubblewrap and
strace. No credential, host home, external network or fake Codex executable is used.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import tempfile

from acquire_codex_pilot_protocol import BINARY_SHA256, digest
from observe_codex_pilot_context import PROTOCOL, PROTOCOL_SHA256, expected_config, namespace_command
from codex_native_context_guard import (LIMIT, SELECTORS, VARIANTS, initial_files, read_trace,
                                        source_binding, validate_child, validate_initial_root, config_refusals)

BASELINE = "cli_manifests/codex/reports/0.156.1/pilot-runtime-context.json"
BASELINE_SHA256 = "fa355ec4e9f841e1a9ecb236e688771c0884be344ecb3936feb8c64bc0d410cf"


def file_digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(65536), b""):
            value.update(chunk)
    return value.hexdigest()


def native_manifest(root):
    entries = []
    for path in sorted(root.rglob("*")):
        relative = str(path.relative_to(root))
        if relative == "trace" or relative.startswith("trace/"):
            continue  # Observer output is recorded separately, not attributed to Codex.
        mode = path.lstat().st_mode
        entry = {"path": relative, "mode": stat.S_IMODE(mode)}
        if stat.S_ISLNK(mode):
            entry.update(kind="symlink", target=os.readlink(path))
        elif stat.S_ISREG(mode):
            entry.update(kind="file", sha256=file_digest(path))
        elif stat.S_ISDIR(mode):
            entry.update(kind="directory")
        else:
            entry.update(kind="other")
        entries.append(entry)
    return entries


def build_driver(root):
    result = subprocess.run(["cargo", "build", "--locked", "-p", "xtask", "--example",
                             "codex_native_context", "--message-format=json"], cwd=root,
                            check=True, stdout=subprocess.PIPE, timeout=600)
    artifacts = [json.loads(line) for line in result.stdout.splitlines() if line.startswith(b"{")]
    executables = [a["executable"] for a in artifacts if a.get("reason") == "compiler-artifact"
                   and a.get("target", {}).get("name") == "codex_native_context" and a.get("executable")]
    if len(executables) != 1:
        raise ValueError("missing or ambiguous built producer artifact")
    return Path(executables[0]).resolve(strict=True)


def dependencies(programs):
    libraries = set()
    for program in programs:
        output = subprocess.check_output(["ldd", str(program)], text=True, timeout=10)
        if "not found" in output:
            raise ValueError("unresolved producer/tracer dependency")
        libraries.update(re.findall(r"/[^\s()]+", output))
    if not libraries:
        raise ValueError("expected explicit GNU producer/tracer runtime libraries")
    return sorted(Path(path) for path in libraries)


def run_case(bwrap, binary, driver, tracer, libraries, config, selector, variant, raw_traces=None):
    with tempfile.TemporaryDirectory(prefix="codex-adapter-") as temporary:
        root = Path(temporary)
        for name in ("home", "work/.git/objects", "work/.git/refs/heads", "tmp", "xdg/config",
                     "xdg/data", "xdg/cache", "trace"):
            (root / name).mkdir(parents=True, exist_ok=True)
        for name, content in initial_files(config).items():
            (root / name).write_bytes(content)
        validate_initial_root(root, config)
        before = native_manifest(root)
        command, environment = namespace_command(bwrap, binary, root)
        # Leave generation of these two variables to the real wrapper, not the producer.
        command = command[:-1] + ["--unsetenv", "CODEX_BINARY", "--unsetenv", "RUST_LOG"]
        environment = {k: v for k, v in environment.items() if k not in ("CODEX_BINARY", "RUST_LOG")}
        environment["PWD"] = "/pilot/work"  # bubblewrap's --chdir sets this.
        for library in libraries:
            command += ["--ro-bind", str(library), str(library)]
        command += ["--ro-bind", str(driver), "/bin/adapter-observe", "--ro-bind", str(tracer), "/bin/strace",
                    "/bin/strace", "-ff", "-qq", "-v", "-xx", "-s", "65536", "-e",
                    "trace=execve,chdir,fchdir,clone,clone3,fork,vfork", "-o", "/pilot/trace/trace",
                    "/bin/adapter-observe", selector, variant]
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            stdout, stderr = process.communicate(timeout=20)
        except subprocess.TimeoutExpired:
            process.kill()  # Death of the namespace owner reaps/kills its private process tree.
            process.communicate()
            raise ValueError("producer namespace exceeded its owned deadline")
        if process.returncode or len(stdout) + len(stderr) > LIMIT:
            raise ValueError("producer failed or exceeded output limit")
        result = json.loads(stdout)
        transport = "app-server" if variant == "interactive" or selector.startswith("fork") else "exec"
        if (result.get("selector") != selector or result.get("variant") != variant
                or result.get("transport") != transport or result.get("execution_verified") is not False):
            raise ValueError("producer returned another request or unsupported execution claim")
        child, descendants = read_trace(root / "trace")
        if child["producer_environment"] != environment:
            raise ValueError("producer inherited an unbound environment")
        validate_child(child, selector, variant, environment)
        with (root / "home/config.toml").open("rb") as stream:
            config_after = stream.read(LIMIT + 1)
        if len(config_after) > LIMIT:
            raise ValueError("native config exceeds observation limit")
        # Retain negative observations. A changed config cannot become a passed
        # context merely because the adapter's initial execve matched.
        refusals = config_refusals(config, config_after)
        traces = [{"file": path.name, "sha256": file_digest(path), "bytes": path.stat().st_size}
                  for path in sorted((root / "trace").glob("trace.*"))]
        if raw_traces is not None:
            destination = raw_traces / (selector + "-" + variant)
            destination.mkdir(parents=True, exist_ok=False)
            for path in (root / "trace").glob("trace.*"):
                shutil.copyfile(path, destination / path.name)
        return {"request": {"selector": selector, "variant": variant}, "adapter": result,
                "child": child, "descendant_exec_attempts": descendants,
                "before": before, "after": native_manifest(root), "traces": traces,
                "config_after_toml": config_after.decode(), "context_refusals": refusals,
                "diagnostics": stderr.decode(errors="replace"), "child_boundary_observed": True,
                "context_complete": False, "execution_verified": False}


def observe(root, binary, tracer, bwrap, raw_traces=None):
    source = source_binding(root)
    protocol, baseline = (root / PROTOCOL).read_bytes(), (root / BASELINE).read_bytes()
    if (digest(protocol) != PROTOCOL_SHA256 or digest(baseline) != BASELINE_SHA256
            or file_digest(binary) != BINARY_SHA256):
        raise ValueError("native binary or reviewed context acquisition differs from its pin")
    config, unbound = expected_config(json.loads(protocol))
    if json.loads(baseline)["expected_config_toml"] != config:
        raise ValueError("baseline and producer configuration disagree")
    driver = build_driver(root)
    libraries = dependencies([driver, tracer])
    inputs = [binary, driver, tracer, Path(bwrap), *libraries,
              *(root / "scripts" / name for name in ("observe_codex_adapter_children.py",
                 "codex_native_context_guard.py", "observe_codex_pilot_context.py", "acquire_codex_pilot_protocol.py"))]
    hashes = {str(path): file_digest(path) for path in inputs}
    recorded_hashes = {"native_codex": hashes[str(binary)], "built_adapter_producer": hashes[str(driver)],
                       "strace": hashes[str(tracer)], "bubblewrap": hashes[str(Path(bwrap))]}
    recorded_hashes.update({"gnu_runtime:" + str(path): hashes[str(path)] for path in libraries})
    recorded_hashes.update({str(path.relative_to(root)): hashes[str(path)] for path in inputs if root in path.parents})
    cases = []
    for selector in SELECTORS:
        for variant in VARIANTS:
            cases.append(run_case(bwrap, binary, driver, tracer, libraries, config, selector, variant, raw_traces))
            print(f"Observed {selector}/{variant} (no verified execution)", flush=True)
    if source_binding(root) != source or any(file_digest(Path(path)) != sha for path, sha in hashes.items()):
        raise ValueError("producer source, native binary or launch inputs changed during observation")
    return {"schema_version": 1, "version": "0.156.1", "target": "x86_64-unknown-linux-musl",
            "source": source, "input_sha256": recorded_hashes, "protocol_sha256": PROTOCOL_SHA256,
            "reviewed_baseline_sha256": BASELINE_SHA256, "expected_config_toml": config,
            "unbound_object_feature_defaults": unbound, "cases": cases, "packet_binding": None,
            "context_complete": False, "execution_verified": False,
            "remaining": ["tool_registry and other unexposed effective defaults",
                          "native descendant/loader/provider effects and positive execution context",
                          "live native turns, approval execution and independent adequacy/adoption",
                          "individual classification approval, packet binding and isolated chain proof"],
            "limits": "Offline actual adapter spawn audit; no P/O/E admission, A1 attestation, qualification or production enrollment. Tracing changes timing. Failed helper attempts and native errors/timeouts remain visible."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--strace", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--raw-traces", type=Path, help="optional private directory for complete bounded syscall traces")
    args = parser.parse_args()
    bwrap = shutil.which("bwrap")
    if not bwrap:
        parser.error("bubblewrap is required; no unisolated fallback")
    root = Path(__file__).resolve().parents[1]
    report = observe(root, args.binary.resolve(strict=True), args.strace.resolve(strict=True), bwrap, args.raw_traces)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(digest(args.output.read_bytes()))


if __name__ == "__main__":
    main()
