import hashlib
import io
from pathlib import Path
import stat
import tarfile
import tempfile
import unittest
import warnings
import zipfile
import depth_ci_archive as archive

class ArchiveTests(unittest.TestCase):
    def zipped(self, entries):
        raw=io.BytesIO()
        with warnings.catch_warnings():
            warnings.simplefilter('ignore',UserWarning)
            with zipfile.ZipFile(raw,'w',compression=zipfile.ZIP_DEFLATED) as output:
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
