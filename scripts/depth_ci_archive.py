#!/usr/bin/env python3
"""Bounded archive validation, not a Git reader or a source of verification authority."""
import hashlib
import io
import json
from pathlib import Path
import stat
import sys
import tarfile
import zipfile

MAX=1048576

def registry(archive, source, expected):
    with Path(archive).open("rb") as stream:
        raw=stream.read(64*MAX+1)
    if len(raw)>64*MAX or hashlib.sha256(raw).hexdigest()!=expected:
        raise ValueError('registry archive identity mismatch')
    directory=Path(source).resolve()
    expected_files=set()
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r|gz') as tar:
        expanded=0
        for count,member in enumerate(tar,1):
            expanded+=member.size
            if count>16384 or expanded>256*MAX:
                raise ValueError('registry archive bounds exceeded')
            parts=member.name.split('/')
            if len(parts)<2 or any(p in ('','..','.') or '\\' in p for p in parts):
                raise ValueError('unsafe registry entry')
            relative='/'.join(parts[1:]); full=directory/relative
            if member.isdir():
                continue
            if not member.isfile() or relative in expected_files or full.is_symlink():
                raise ValueError('unsupported registry entry')
            expected_files.add(relative)
            for parent in full.parents:
                if parent==directory:break
                if parent.is_symlink():raise ValueError('registry symlink ancestor')
            if full.read_bytes()!=tar.extractfile(member).read():
                raise ValueError('registry source differs from locked archive')
            if bool(full.stat().st_mode&stat.S_IXUSR)!=bool(member.mode&stat.S_IXUSR):
                raise ValueError('registry source mode differs from locked archive')
    actual={p.relative_to(directory).as_posix() for p in directory.rglob('*') if p.is_file() or p.is_symlink()}
    if actual-expected_files-{'.cargo-ok','.cargo-checksum.json'} or expected_files-actual:
        raise ValueError('registry source inventory differs from archive')
    return {'verified':expected}

def witness(raw):
    if len(raw)>2*MAX:raise ValueError('artifact archive exceeds bounds')
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        entries=archive.infolist()
        if len(entries)!=1 or entries[0].filename!='witness.json' or entries[0].file_size>MAX:
            raise ValueError('artifact must contain exactly one bounded witness.json')
        entry=entries[0]
        if entry.flag_bits&1 or stat.S_IFMT(entry.external_attr>>16) not in (0,stat.S_IFREG):
            raise ValueError('unsupported artifact entry')
        def unique(pairs):
            result={}
            for key,value in pairs:
                if key in result:raise ValueError('duplicate witness JSON member')
                result[key]=value
            return result
        value=json.loads(archive.read(entry),object_pairs_hook=unique)
        if not isinstance(value,dict):raise ValueError('witness requires an object')
        return value

if __name__=='__main__':
    try:
        if sys.argv[1]=='registry': value=registry(*sys.argv[2:])
        elif sys.argv[1]=='witness':value=witness(sys.stdin.buffer.read(2*MAX+1))
        else:raise ValueError('unknown archive operation')
        json.dump(value,sys.stdout,separators=(',',':'))
    except (ValueError,OSError,KeyError,tarfile.TarError,zipfile.BadZipFile) as error:
        print(str(error),file=sys.stderr)
        sys.exit(1)
