import hashlib
import io
from pathlib import Path
import stat
import struct
import tarfile
import tempfile
import unittest
import warnings
import zipfile
import zlib
import depth_ci_archive as archive

class ArchiveTests(unittest.TestCase):
    def zipped(self, entries, compression=zipfile.ZIP_DEFLATED):
        raw=io.BytesIO()
        with warnings.catch_warnings():
            warnings.simplefilter('ignore',UserWarning)
            with zipfile.ZipFile(raw,'w',compression=compression) as output:
                for name,data in entries:output.writestr(name,data)
        return raw.getvalue()
    def test_exact_witness(self):
        self.assertEqual(archive.witness(self.zipped([('witness.json','{"schema_version":1}')])),{'schema_version':1})
    def test_duplicate_extra_traversal_and_oversized_entries_refuse(self):
        for entries in [[('witness.json','{}'),('witness.json','{}')],[('witness.json','{}'),('extra','{}')],
            [('../witness.json','{}')],[('witness.json','x'*(archive.MAX+1))]]:
            with self.subTest(entries=[e[0] for e in entries]),self.assertRaises(ValueError):archive.witness(self.zipped(entries))
    def test_duplicate_json_members_refuse(self):
        for value in ['{"x":1,"x":2}','{"outer":{"x":1,"x":2}}']:
            with self.assertRaises(ValueError):archive.witness(self.zipped([('witness.json',value)]))
    def test_non_regular_entry_refuses(self):
        for mode in [stat.S_IFLNK,stat.S_IFDIR,stat.S_IFIFO]:
            item=zipfile.ZipInfo('witness.json');item.external_attr=(mode|0o644)<<16
            raw=io.BytesIO()
            with zipfile.ZipFile(raw,'w') as output:output.writestr(item,'{}')
            with self.assertRaises(ValueError):archive.witness(raw.getvalue())
    def test_in1_exact_single_control_entry_and_64k_boundary(self):
        # The archive reader returns bytes; canonical JSON belongs to the Rust parser.
        value=b'x'*65536
        for method in [zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED,zipfile.ZIP_BZIP2,zipfile.ZIP_LZMA]:
            with self.subTest(method=method):
                self.assertEqual(archive.in1(self.zipped([('admission.json',value)],method)),value)
                self.assertEqual(archive.in1(self.zipped([('admission.json',b'')],method)),b'')
        for entries in [[], [('admission.json','{}'),('admission.json','{}')],
            [('admission.json','{}'),('extra','{}')], [('../admission.json','{}')],
            [('witness.json','{}')], [('admission.json','x'*65537)]]:
            with self.subTest(entries=[e[0] for e in entries]),self.assertRaises(ValueError):
                archive.in1(self.zipped(entries))
        with self.assertRaises(ValueError):archive.in1(b'x'*(2*archive.MAX+1))
    def test_in1_non_regular_and_encrypted_entries_refuse(self):
        for mode in [stat.S_IFLNK,stat.S_IFDIR,stat.S_IFIFO]:
            item=zipfile.ZipInfo('admission.json');item.external_attr=(mode|0o644)<<16
            raw=io.BytesIO()
            with zipfile.ZipFile(raw,'w') as output:output.writestr(item,'{}')
            with self.assertRaises(ValueError):archive.in1(raw.getvalue())
        raw=bytearray(self.zipped([('admission.json','{}')]))
        central=raw.index(b'PK\x01\x02');raw[central+8]|=1
        with self.assertRaises(ValueError):archive.in1(bytes(raw))
    def test_in1_size_lie_with_valid_prefix_crc_refuses(self):
        raw=bytearray(self.zipped([('admission.json',b'{}'+b'x'*100000)]))
        self.assertEqual(len(raw),243)
        central=raw.index(b'PK\x01\x02')
        for crc_offset,size_offset in [(14,22),(central+16,central+24)]:
            struct.pack_into('<I',raw,crc_offset,zlib.crc32(b'{}'))
            struct.pack_into('<I',raw,size_offset,2)
        # ZipExtFile discards the suffix and accepts the CRC of just the declared prefix.
        with zipfile.ZipFile(io.BytesIO(raw)) as zipped:
            self.assertEqual(zipped.read('admission.json'),b'{}')
        with self.assertRaises(ValueError):archive.in1(bytes(raw))
    def test_in1_actual_length_completion_and_crc_must_agree(self):
        for method in [zipfile.ZIP_STORED,zipfile.ZIP_DEFLATED,zipfile.ZIP_BZIP2,zipfile.ZIP_LZMA]:
            for mutation in ['short_size','long_size','crc','unfinished','excess_compressed']:
                with self.subTest(method=method,mutation=mutation):
                    output=io.BytesIO()
                    with zipfile.ZipFile(output,'w',compression=method) as zipped:
                        zipped.writestr('admission.json',b'{}')
                    raw=bytearray(output.getvalue())
                    central=raw.index(b'PK\x01\x02')
                    if mutation in ['short_size','long_size']:
                        size=1 if mutation=='short_size' else 3
                        struct.pack_into('<I',raw,22,size)
                        struct.pack_into('<I',raw,central+24,size)
                    elif mutation=='crc':
                        struct.pack_into('<I',raw,14,0)
                        struct.pack_into('<I',raw,central+16,0)
                    else:
                        size=struct.unpack_from('<I',raw,18)[0]
                        size+=-1 if mutation=='unfinished' else 1
                        struct.pack_into('<I',raw,18,size)
                        struct.pack_into('<I',raw,central+20,size)
                    with self.assertRaises((ValueError,zipfile.BadZipFile)):archive.in1(bytes(raw))
    def test_in1_data_descriptor_stream_refuses_size_lie(self):
        class Unseekable(io.BytesIO):
            def seek(self,*args):raise OSError('stream has no seek')
        for size in [65536,100002]:
            output=Unseekable()
            value=b'{}'+b'x'*(size-2)
            with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED) as zipped:
                zipped.writestr('admission.json',value)
            raw=bytearray(output.getvalue())
            if size==65536:
                self.assertEqual(archive.in1(bytes(raw)),value)
            else:
                central=raw.index(b'PK\x01\x02')
                descriptor=raw.index(b'PK\x07\x08')
                struct.pack_into('<I',raw,central+16,zlib.crc32(b'{}'))
                struct.pack_into('<I',raw,central+24,2)
                struct.pack_into('<I',raw,descriptor+4,zlib.crc32(b'{}'))
                struct.pack_into('<I',raw,descriptor+12,2)
                with self.assertRaises(ValueError):archive.in1(bytes(raw))
    def descriptor_archive(self,value,zip64,signature):
        class Unseekable(io.BytesIO):
            def seek(self,*args):raise OSError('stream has no seek')
        output=Unseekable()
        with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED) as zipped:
            with zipped.open('admission.json','w',force_zip64=zip64) as entry:
                entry.write(value)
        raw=bytearray(output.getvalue())
        descriptor=raw.index(b'PK\x07\x08')
        if not signature:
            del raw[descriptor:descriptor+4]
            central=raw.index(b'PK\x01\x02')
            struct.pack_into('<I',raw,raw.index(b'PK\x05\x06')+16,central)
        return raw,descriptor
    def test_in1_required_descriptors_match_crc_and_sizes(self):
        for zip64 in [False,True]:
            for signature in [False,True]:
                for value in [b'{}',b'x'*65536]:
                    raw,_=self.descriptor_archive(value,zip64,signature)
                    self.assertEqual(archive.in1(bytes(raw)),value)
                for mutation in ['crc','compressed','expanded','garbage','missing','truncated']:
                    with self.subTest(zip64=zip64,signature=signature,mutation=mutation):
                        raw,start=self.descriptor_archive(b'{}',zip64,signature)
                        central=raw.index(b'PK\x01\x02')
                        fields=start+(4 if signature else 0)
                        width=8 if zip64 else 4
                        if mutation=='crc':struct.pack_into('<I',raw,fields,0)
                        elif mutation=='compressed':struct.pack_into('<Q' if zip64 else '<I',raw,fields+4,0xffffffff)
                        elif mutation=='expanded':struct.pack_into('<Q' if zip64 else '<I',raw,fields+4+width,1000000)
                        elif mutation=='garbage':raw[start:central]=b'x'*(central-start)
                        else:
                            del raw[start:central if mutation=='missing' else central-1]
                            central=raw.index(b'PK\x01\x02')
                            struct.pack_into('<I',raw,raw.index(b'PK\x05\x06')+16,central)
                        with self.assertRaises(ValueError):archive.in1(bytes(raw))
    def test_in1_zip64_local_sizes_and_excess_compressed_data(self):
        output=io.BytesIO()
        with zipfile.ZipFile(output,'w',compression=zipfile.ZIP_DEFLATED) as zipped:
            with zipped.open('admission.json','w',force_zip64=True) as entry:
                entry.write(b'x'*65536)
        self.assertEqual(archive.in1(output.getvalue()),b'x'*65536)
        local_zip64=bytearray(output.getvalue())
        struct.pack_into('<H',local_zip64,4,45)
        struct.pack_into('<II',local_zip64,18,0xffffffff,0xffffffff)
        self.assertEqual(archive.in1(bytes(local_zip64)),b'x'*65536)
        raw=bytearray(self.zipped([('admission.json',b'{}')]))
        central=raw.index(b'PK\x01\x02')
        raw[central:central]=b'excess'
        central+=6
        compressed=struct.unpack_from('<I',raw,18)[0]+6
        struct.pack_into('<I',raw,18,compressed)
        struct.pack_into('<I',raw,central+20,compressed)
        end=raw.index(b'PK\x05\x06')
        struct.pack_into('<I',raw,end+16,central)
        with zipfile.ZipFile(io.BytesIO(raw)) as zipped:
            self.assertEqual(zipped.read('admission.json'),b'{}')
        with self.assertRaises(ValueError):archive.in1(bytes(raw))
    def package(self,root):
        source=root/'source';(source/'src').mkdir(parents=True);(source/'src/lib.rs').write_bytes(b'pub fn original() {}')
        raw=io.BytesIO()
        with tarfile.open(fileobj=raw,mode='w:gz') as tar:
            info=tarfile.TarInfo('example-1.0.0/src/lib.rs');info.size=len((source/'src/lib.rs').read_bytes());info.mode=0o644
            tar.addfile(info,io.BytesIO((source/'src/lib.rs').read_bytes()))
        path=root/'example.crate';path.write_bytes(raw.getvalue())
        return path,source,hashlib.sha256(raw.getvalue()).hexdigest()
    def test_registry_contents_and_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            path,source,digest=self.package(Path(directory))
            self.assertEqual(archive.registry(path,source,digest),{'verified':digest})
            (source/'.cargo-ok').write_text('cache metadata')
            self.assertEqual(archive.registry(path,source,digest),{'verified':digest})
            (source/'unexpected.rs').write_text('changed')
            with self.assertRaises(ValueError):archive.registry(path,source,digest)
    def test_registry_tamper_wrong_digest_and_mode(self):
        for mutation in ['bytes','checksum','mode','symlink']:
            with self.subTest(mutation=mutation),tempfile.TemporaryDirectory() as directory:
                path,source,digest=self.package(Path(directory));file=source/'src/lib.rs'
                if mutation=='bytes':file.write_text('changed')
                if mutation=='checksum':digest='0'*64
                if mutation=='mode':file.chmod(0o755)
                if mutation=='symlink':file.unlink();file.symlink_to(path)
                with self.assertRaises(ValueError):archive.registry(path,source,digest)

if __name__=='__main__':unittest.main()
