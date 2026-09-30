"""Package set, architecture, pathname and hash boundary tests."""
import hashlib
import json
import pathlib
import tempfile
import unittest

from inputs import InputError, package_rows, verify_packages


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)

    def package(self):
        raw = b'pinned-package'
        row = {'file': 'p.deb', 'architecture': 'arm64', 'bytes': len(raw),
               'sha256': hashlib.sha256(raw).hexdigest()}
        (self.root / 'p.deb').write_bytes(raw)
        (self.root / '01.json').write_text(json.dumps([row]))
        return row

    def test_tampered_package(self):
        row = self.package()
        (self.root / 'p.deb').write_bytes(b'tampered')
        with self.assertRaises(InputError):
            verify_packages(self.root, [row])

    def test_unlocked_extra_package(self):
        row = self.package()
        (self.root / 'extra.deb').write_bytes(b'extra')
        with self.assertRaises(InputError):
            verify_packages(self.root, [row])

    def test_package_traversal(self):
        row = self.package()
        row['file'] = '../p.deb'
        (self.root / '01.json').write_text(json.dumps([row]))
        with self.assertRaises(InputError):
            package_rows(self.root)

    def test_foreign_architecture(self):
        row = self.package()
        row['architecture'] = 'amd64'
        (self.root / '01.json').write_text(json.dumps([row]))
        with self.assertRaises(InputError):
            package_rows(self.root)


if __name__ == "__main__":
    unittest.main()
