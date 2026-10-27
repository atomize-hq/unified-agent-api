"""Finite guards for offline adapter-spawn observations; never evidence admission."""
import json
from pathlib import Path
import re
import subprocess

LIMIT = 2 * 1024 * 1024
SELECTORS = ("initial-json", "resume-last-json", "resume-id-json", "fork-last-stdio", "fork-id-stdio")
VARIANTS = ("unattended", "interactive", "external")
SESSION = "11111111-1111-4111-8111-111111111111"


def parse_exec(line):
    def decode(match):
        if not re.fullmatch(r"(?:\\x[0-9a-f]{2})*", match[1]):
            raise ValueError("trace strings must be complete hexadecimal bytes")
        return json.dumps(bytes.fromhex(match[1].replace("\\x", "")).decode("utf-8"))
    text = re.sub(r'"((?:[^"\\]|\\.)*)"', decode, line)
    if not text.startswith("execve("):
        raise ValueError("expected execve")
    decoder, values, position = json.JSONDecoder(), [], len("execve(")
    for index in range(3):
        value, position = decoder.raw_decode(text, position)
        values.append(value)
        if index < 2:
            if text[position:position + 2] != ", ":
                raise ValueError("incomplete execve arguments")
            position += 2
    tail = text[position:]
    if not re.fullmatch(r"\) = (0|-1 [A-Z0-9_]+(?: \(.*\))?)", tail):
        raise ValueError("incomplete execve result")
    executable, argv, raw_environment = values
    if not isinstance(executable, str) or not all(isinstance(v, list) for v in (argv, raw_environment)):
        raise ValueError("invalid execve shape")
    if not all(isinstance(s, str) and "\0" not in s for s in argv + raw_environment):
        raise ValueError("invalid execve strings")
    environment = {}
    for entry in raw_environment:
        key, separator, value = entry.partition("=")
        if not separator or not key or key in environment:
            raise ValueError("invalid or duplicate environment key")
        environment[key] = value
    return {"executable": executable, "argv": argv, "environment": environment,
            "success": tail == ") = 0", "result": tail[4:]}


def read_trace(root):
    files = sorted(root.glob("trace.*"))
    if not files or len(files) > 512 or sum(p.stat().st_size for p in files) > LIMIT:
        raise ValueError("missing or oversized trace")
    events, parents = [], {}
    for path in files:
        pid, cwd = int(path.suffix[1:]), None
        for line in path.read_text().splitlines():
            if line.startswith("chdir(") and line.endswith(") = 0"):
                raw = line[len("chdir("):-len(") = 0")]
                # Reuse the strict byte decoder for the cwd string too.
                cwd = parse_exec(f"execve({raw}, [], []) = 0")["executable"]
            elif line.startswith("fchdir("):
                raise ValueError("unresolved fd-based cwd")
            elif line.startswith("execve("):
                events.append(dict(parse_exec(line), pid=pid, cwd=cwd))
            elif line.startswith(("clone(", "clone3(", "fork(", "vfork(")):
                match = re.search(r"\) = ([0-9]+)$", line)
                if match:
                    child = int(match[1])
                    if child in parents:
                        raise ValueError("ambiguous child ownership")
                    parents[child] = pid
    producers = [e for e in events if e["executable"] == "/bin/adapter-observe" and e["success"]]
    if len(producers) != 1:
        raise ValueError("missing or ambiguous producer process")
    producer = producers[0]
    children = [e for e in events if e["executable"] == "/bin/codex"
                and parents.get(e["pid"]) == producer["pid"]]
    if len(children) != 1:
        raise ValueError("missing or ambiguous direct adapter child")
    child = dict(children[0], parent_pid=producer["pid"], producer_environment=producer["environment"])
    descendants = [e for e in events if e is not producer and e is not children[0]]
    return child, descendants


def validate_child(child, selector, variant, producer_environment):
    if selector not in SELECTORS or variant not in VARIANTS:
        raise ValueError("unknown fixed request")
    expected_environment = dict(producer_environment, CODEX_BINARY="/bin/codex", RUST_LOG="error")
    if (child["executable"] != "/bin/codex" or not child["success"]
            or child["cwd"] != "/pilot/work" or child["environment"] != expected_environment):
        raise ValueError("foreign child executable, cwd, environment or unsuccessful exec")
    if variant == "interactive" or selector.startswith("fork"):
        sandbox = "danger-full-access" if variant == "external" else "workspace-write"
        expected = ["/bin/codex", "--config", f'sandbox_mode="{sandbox}"', "app-server"]
    else:
        prefix = (["--dangerously-bypass-approvals-and-sandbox"] if variant == "external" else
                  ["--config", 'approval_policy="never"', "--ask-for-approval", "never",
                   "--sandbox", "workspace-write"])
        expected = ["/bin/codex"] + prefix + ["exec", "--color", "never", "--skip-git-repo-check",
                                               "--json", "--output-last-message", "<private-output>"]
        if selector.startswith("resume"):
            expected += ["resume"] + (["--last"] if "-last-" in selector else [SESSION]) + ["-"]
    actual = list(child["argv"])
    if "--output-last-message" in actual:
        index = actual.index("--output-last-message") + 1
        if index >= len(actual) or not re.fullmatch(r"/pilot/tmp/codex_last_message_[0-9]+_[0-9]+\.txt", actual[index]):
            raise ValueError("unbound output path")
        actual[index] = "<private-output>"
    if actual != expected:
        raise ValueError("child invocation differs from the fixed request regime")


def source_binding(root):
    def git(*args):
        return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()
    if git("status", "--porcelain"):
        raise ValueError("native producer requires a clean committed source tree")
    return {"commit": git("rev-parse", "HEAD"), "tree": git("rev-parse", "HEAD^{tree}")}


def initial_files(config):
    return {"home/config.toml": config.encode(), "work/.git/HEAD": b"ref: refs/heads/pilot\n",
            "work/.git/config": b"[core]\nrepositoryformatversion = 0\nbare = false\n"}


def validate_initial_root(root, config):
    expected = initial_files(config)
    allowed_dirs = {"home", "work", "work/.git", "work/.git/objects", "work/.git/refs",
                    "work/.git/refs/heads", "tmp", "xdg", "xdg/config", "xdg/data", "xdg/cache", "trace"}
    found = {}
    for path in root.rglob("*"):
        relative = str(path.relative_to(root))
        if path.is_symlink() or (path.is_dir() and relative not in allowed_dirs):
            raise ValueError("foreign loader/root or symlink in initial context")
        if path.is_file():
            found[relative] = path.read_bytes()
    if found != expected:
        raise ValueError("foreign or changed initial configuration/project input")
