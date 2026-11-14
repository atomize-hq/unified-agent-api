#!/usr/bin/env python3
"""Bounded archive validation, not a Git reader or a source of verification authority."""
import bz2
import hashlib
import io
import json
import lzma
from pathlib import Path
import stat
import struct
import sys
import tarfile
import zipfile
import zlib

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

def in1(raw):
    """Decode transport only; Rust owns canonical JSON parsing and authentication."""
    if len(raw)>2*MAX:raise ValueError('control archive exceeds bounds')
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        entries=archive.infolist()
        if len(entries)!=1 or entries[0].filename!='admission.json' or entries[0].file_size>65536:
            raise ValueError('expected one bounded admission.json')
        entry=entries[0]
        if entry.flag_bits&1 or stat.S_IFMT(entry.external_attr>>16) not in (0,stat.S_IFREG):
            raise ValueError('unsupported control entry')
        # Retain ZipFile's local-header/name/overlap checks without reading ZipExtFile.
        # its reader truncates decompressed bytes to the untrusted declared file_size.
        with archive.open(entry):
            pass
        header=struct.unpack_from('<4s5H3I2H',raw,entry.header_offset)
        if header[2]!=entry.flag_bits or header[3]!=entry.compress_type:
            raise ValueError('control local and central headers disagree')
        local=list(header[6:9])
        start=entry.header_offset+30+header[9]+header[10]
        if 0xffffffff in local[1:]:
            extra=raw[start-header[10]:start]
            while len(extra)>=4:
                kind,size=struct.unpack_from('<HH',extra)
                data=extra[4:4+size]
                if len(data)!=size:
                    raise ValueError('truncated local ZIP64 field')
                if kind==1:
                    # ZIP64 stores expanded size before compressed size.
                    for index in [2,1]:
                        if local[index]==0xffffffff:
                            if len(data)<8:
                                raise ValueError('missing local ZIP64 size')
                            local[index]=struct.unpack_from('<Q',data)[0]
                            data=data[8:]
                    break
                extra=extra[4+size:]
        for actual,expected in zip(local,(entry.CRC,entry.compress_size,entry.file_size)):
            if actual!=expected and not (entry.flag_bits&8 and actual==0):
                raise ValueError('control local and central sizes or CRC disagree')
        compressed=raw[start:start+entry.compress_size]
        value=control_payload(compressed,entry.compress_type)
        if len(value)>65536 or len(value)!=entry.file_size:
            raise ValueError('control payload exceeds bounds or declared size')
        if zlib.crc32(value)!=entry.CRC:
            raise ValueError('control CRC mismatch')
        return value

def control_payload(compressed,method):
    """Bound native streaming decompressors independently of ZIP's expanded-size fields."""
    if method==zipfile.ZIP_STORED:
        if len(compressed)>65536:
            raise ValueError('control payload exceeds 64 KiB')
        return compressed
    if method==zipfile.ZIP_DEFLATED:
        decoder=zlib.decompressobj(-15)
    elif method==zipfile.ZIP_BZIP2:
        decoder=bz2.BZ2Decompressor()
    elif method==zipfile.ZIP_LZMA:
        # ZIP stores SDK version + property length + LZMA1 properties. An ALONE
        # header uses the same five properties followed by an unknown expanded size.
        if len(compressed)<9 or compressed[2:4]!=b'\x05\x00':
            raise ValueError('invalid control LZMA header')
        compressed=compressed[4:9]+b'\xff'*8+compressed[9:]
        decoder=lzma.LZMADecompressor(format=lzma.FORMAT_ALONE)
    else:
        raise ValueError('unsupported control compression')
    value=bytearray()
    try:
        for offset in range(0,len(compressed),4096):
            block=compressed[offset:offset+4096]
            value.extend(decoder.decompress(block,max_length=65537-len(value)))
            if len(value)>65536:
                raise ValueError('control payload exceeds 64 KiB')
            if decoder.eof:
                if decoder.unused_data or offset+len(block)!=len(compressed):
                    raise ValueError('excess control compressed data')
                break
    except (OSError,EOFError,zlib.error,lzma.LZMAError) as error:
        raise ValueError('invalid control compressed data') from error
    if not decoder.eof:
        raise ValueError('incomplete control compressed data')
    return bytes(value)

if __name__=='__main__':
    try:
        if sys.argv[1]=='registry': value=registry(*sys.argv[2:])
        elif sys.argv[1]=='in1':
            sys.stdout.buffer.write(in1(sys.stdin.buffer.read(2*MAX+1)))
            sys.exit(0)
        elif sys.argv[1]=='witness':value=witness(sys.stdin.buffer.read(2*MAX+1))
        else:raise ValueError('unknown archive operation')
        json.dump(value,sys.stdout,separators=(',',':'))
    except (ValueError,OSError,KeyError,tarfile.TarError,zipfile.BadZipFile) as error:
        print(str(error),file=sys.stderr)
        sys.exit(1)
