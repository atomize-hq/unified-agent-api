#!/usr/bin/env python3
"""Exact-file staging in disposable maintenance workflow checkouts; no route admission."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess

PUBLICATION_PATHS = (
    'cli_manifests/support_matrix/current.json',
    'docs/specs/unified-agent-api/support-matrix.md',
    'crates/agent_api/src/runtime_support_data.rs',
)


def git(*args):
    return subprocess.check_output(['git', '--literal-pathspecs', *args])


def paths(values):
    result = set()
    for value in values:
        if (not isinstance(value, str) or not value or value.startswith('/')
                or any(c in value for c in '\\*?[]\r\n\0')
                or any(p in ('', '.', '..', '.git') for p in value.split('/'))):
            raise ValueError(f'not an exact repository file path: {value!r}')
        result.add(value)
    return sorted(result)


def fingerprint(path):
    path = paths([path])[0]
    current = Path.cwd()
    for part in PurePosixPath(path).parts:
        current /= part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError:
            return None
        if stat.S_ISLNK(mode):
            raise ValueError(f'symlink in delivery path: {path}')
    if not stat.S_ISREG(mode):
        raise ValueError(f'expected regular delivery file: {path}')
    return hashlib.sha256(current.read_bytes()).hexdigest()


def changed(staged=False):
    args = ['diff', '--name-only', '-z']
    if staged:
        args += ['--cached']
    return set(git(*args, 'HEAD', '--').decode().rstrip('\0').split('\0')) - {''}


def index():
    entries = {}
    for row in git('ls-files', '--stage', '-z').split(b'\0'):
        if not row:
            continue
        info, path = row.split(b'\t', 1)
        mode, oid, stage = info.decode().split()
        if stage != '0':
            raise ValueError('unmerged index; disposable checkout required')
        entries[path.decode()] = (mode, oid)
    return entries


def check_files(files, staged=False):
    entries = index()
    if changed(True) - files.keys():
        raise ValueError('unexpected staged paths; refusing to repair the index')
    for path, expected in files.items():
        if fingerprint(path) != expected:
            raise ValueError(f'checked file changed or missing: {path}')
        if staged:
            entry = entries.get(path)
            if expected is None:
                if entry is not None:
                    raise ValueError(f'absent file remains staged: {path}')
            elif (entry is None or entry[0] not in ('100644', '100755')
                  or hashlib.sha256(git('cat-file', 'blob', entry[1])).hexdigest() != expected):
                raise ValueError(f'staged bytes changed: {path}')


def stage(files):
    tracked = index()
    selected = [p for p in files if files[p] is not None or p in tracked]
    if selected:
        subprocess.run(['git', '--literal-pathspecs', 'add', '-A', '--', *selected], check=True)
    check_files(files, True)


def save(path, value):
    # Scratch state must never become another workspace writer or an owned delivery path.
    parent = Path(path).parent.resolve(strict=True)
    if parent == Path.cwd().resolve() or Path.cwd().resolve() in parent.parents:
        raise ValueError('transport state must be outside the workspace')
    with (parent / Path(path).name).open('x') as output:
        output.write(json.dumps(value, sort_keys=True, indent=2) + '\n')


def load(path):
    return json.loads(Path(path).read_text())


def stage_inputs(plan, state):
    if changed(True):
        raise ValueError('pre-existing staged changes; disposable checkout required')
    owned = paths(plan['output_paths'])
    inputs = paths(plan['input_paths'])
    if not set(inputs) <= set(owned):
        raise ValueError('freeze inputs must be acquisition-owned files')
    if changed() - set(owned):
        raise ValueError('changed tracked inputs outside acquisition ownership')
    files = {p: fingerprint(p) for p in owned}
    stage({p: files[p] for p in inputs})
    save(state, {'head': git('rev-parse', 'HEAD').decode().strip(),
                 'files': files, 'input_paths': inputs})


def stage_delivery(plan, preparation, input_state, state, read_only):
    if not plan and not read_only and changed(True):
        raise ValueError('pre-existing staged changes; disposable checkout required')
    files = {}
    if plan:
        if input_state:
            before = load(input_state)
            if before['head'] != git('rev-parse', 'HEAD').decode().strip():
                raise ValueError('checkout HEAD changed since acquisition staging')
            if set(before['files']) != set(paths(plan['output_paths'])):
                raise ValueError('acquisition ownership changed')
            files.update(before['files'])
            check_files({p: files[p] for p in before['input_paths']}, True)
        elif not read_only:
            raise ValueError('committing acquisition requires staged input state')
        else:
            files.update({p: fingerprint(p) for p in paths(plan['output_paths'])})
    if preparation is not None:
        if preparation.get('applied') is not True:
            raise ValueError('preparation was not successfully applied')
        prepared = {f['path']: f['sha256'] for f in preparation['files']}
        paths(prepared)
        publication = set(PUBLICATION_PATHS) & prepared.keys()
        if publication and publication != set(PUBLICATION_PATHS):
            raise ValueError('partial preparation publication')
        files.update(prepared)
    if plan:
        # Ordinary acquisition publishes after re-freeze. Depth preparation already supplied
        # all three checked publication hashes; never replace those hashes with recaptured ones.
        for path in PUBLICATION_PATHS:
            if path not in files:
                files[path] = fingerprint(path)
            if files[path] is None:
                raise ValueError(f'missing publication output: {path}')
    if not files:
        raise ValueError('no checked delivery files')
    if read_only:
        for path, expected in files.items():
            if fingerprint(path) != expected:
                raise ValueError(f'checked file changed: {path}')
    else:
        check_files(files)
        if changed() - files.keys():
            raise ValueError('changed tracked inputs outside delivery ownership')
        stage(files)
    save(state, {'head': git('rev-parse', 'HEAD').decode().strip(), 'files': files})


def verify(state):
    value = load(state)
    if value['head'] != git('rev-parse', 'HEAD').decode().strip():
        raise ValueError('checkout HEAD changed since delivery planning')
    check_files(value['files'], True)
    if changed() - value['files'].keys():
        raise ValueError('changed tracked inputs outside delivery ownership')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['stage-inputs', 'stage-delivery', 'verify'])
    parser.add_argument('--plan')
    parser.add_argument('--preparation')
    parser.add_argument('--input-state')
    parser.add_argument('--state', required=True)
    parser.add_argument('--read-only', action='store_true')
    parser.add_argument('--github-output')
    parser.add_argument('--bundle-dir')
    args = parser.parse_args()
    if args.operation == 'stage-inputs':
        stage_inputs(load(args.plan), args.state)
    elif args.operation == 'verify':
        verify(args.state)
    else:
        stage_delivery(load(args.plan) if args.plan else None,
                       load(args.preparation) if args.preparation else None,
                       args.input_state, args.state, args.read_only)
        files = load(args.state)['files']
        if args.bundle_dir:
            bundle = Path(args.bundle_dir)
            bundle.mkdir(parents=True, exist_ok=False)
            for path, digest in files.items():
                if digest is not None:
                    target = bundle / path
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(path, target)
        if args.github_output:
            with open(args.github_output, 'a') as output:
                output.write('paths<<MAINTENANCE_DELIVERY_PATHS\n')
                output.write(''.join(p + '\n' for p in files if files[p] is not None))
                output.write('MAINTENANCE_DELIVERY_PATHS\n')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
