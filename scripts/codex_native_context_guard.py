"""Finite guards for offline adapter-spawn observations; never evidence admission."""
import json
from pathlib import Path
import re
import subprocess

LIMIT = 2 * 1024 * 1024
SELECTORS = ("initial-json", "resume-last-json", "resume-id-json", "fork-last-stdio", "fork-id-stdio")
VARIANTS = ("unattended", "interactive", "external")
SESSION = "11111111-1111-4111-8111-111111111111"
PRIVATE_PROJECT_TRUST = '\n[projects."/pilot/work"]\ntrust_level = "trusted"\n'


def config_refusals(before, after):
    # Maintainer approved exactly this native addition for the owned private fixture.
    # Whole bytes are compared: no other project, value, reformat or base change passes.
    return [] if after in (before.encode(), (before + PRIVATE_PROJECT_TRUST).encode()) else ["controlled_config_drift"]


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
    events, parents, unresolved = [], {}, []
    traced_pids = {int(path.suffix[1:]) for path in files}
    for path in files:
        pid, cwd = int(path.suffix[1:]), None
        for number, line in enumerate(path.read_text().splitlines(), 1):
            if line.startswith("chdir("):
                raw, separator, result = line[len("chdir("):].partition(") = ")
                if not separator:
                    raise ValueError("incomplete cwd transition")
                # Reuse the strict byte/result decoder, including failed chdir calls.
                transition = parse_exec(f"execve({raw}, [], []) = {result}")
                if transition["success"]:
                    cwd = transition["executable"]
                    if not cwd.startswith("/"):
                        raise ValueError("unresolved relative cwd transition")
            elif line.startswith("fchdir("):
                raise ValueError("unresolved fd-based cwd")
            elif line.startswith("execve("):
                events.append(dict(parse_exec(line), pid=pid, cwd=cwd))
            elif line.startswith(("clone(", "clone3(", "fork(", "vfork(")):
                match = re.fullmatch(r"(?:clone3?|fork|vfork)\((.*)\) = ([1-9][0-9]*|-1 [A-Z0-9_]+(?: \(.*\))?)", line)
                if not match or any(token in match[1] for token in ("...", "<", ">")):
                    raise ValueError("incomplete process ownership record")
                stack = []
                for char in match[1]:
                    if char in "([{":
                        stack.append(char)
                    elif char in ")]}":
                        if not stack or stack.pop() != {")": "(", "]": "[", "}": "{"}[char]:
                            raise ValueError("malformed process ownership arguments")
                if stack:
                    raise ValueError("incomplete process ownership arguments")
                if not match[2].startswith("-1 "):
                    child = int(match[2])
                    if child in parents:
                        raise ValueError("ambiguous child ownership")
                    parents[child] = pid
            elif line and not line.startswith(("--- ", "+++ ")):
                unresolved.append({"file": path.name, "pid": pid, "line": number, "text": line})
    if not parents.keys() <= traced_pids:
        raise ValueError("missing trace for a successfully spawned process")
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
    return child, descendants, unresolved


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
