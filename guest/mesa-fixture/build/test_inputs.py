"""Host-only adversarial input tests; no Docker, installation, or source execution."""
import hashlib
import io
import json
import pathlib
import tarfile
import tempfile
import unittest

from inputs import InputError, archive_members, load_lock, package_rows, verify_archive, verify_extracted, verify_packages

HERE = pathlib.Path(__file__).resolve().parent


class InputTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)

    def lock(self, change):
        value = json.loads((HERE / 'lock.json').read_text())
        change(value)
        path = self.root / 'lock.json'
        path.write_text(json.dumps(value))
        return path

    def test_locked_recipe(self):
        self.assertEqual(load_lock(HERE / 'lock.json')['mesa']['version'], '25.3.6')

    def test_mutable_base(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value.update(base_image='debian:trixie-slim')))

    def test_missing_driver_fence(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value['meson_args'].remove('-Dvulkan-drivers=virtio')))

    def test_duplicate_fence_override(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value['meson_args'].append('-Dgallium-drivers=llvmpipe')))

    def test_archive_locator_traversal(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value['mesa'].update(archive='../source.tar.gz')))

    def test_separate_option_override(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value['meson_args'].extend(['-D', 'gallium-drivers=swrast'])))

    def test_noncommit_revision(self):
        with self.assertRaises(InputError):
            load_lock(self.lock(lambda value: value['mesa'].update(revision='main')))

    def archive(self, extra):
        stream = io.BytesIO()
        with tarfile.open(fileobj=stream, mode='w') as archive:
            root = tarfile.TarInfo('mesa-pin')
            root.type = tarfile.DIRTYPE
            archive.addfile(root)
            item = tarfile.TarInfo('mesa-pin/regular')
            item.size = 2
            archive.addfile(item, io.BytesIO(b'ok'))
            extra(archive)
        stream.seek(0)
        return tarfile.open(fileobj=stream)

    def reject(self, name, kind=tarfile.REGTYPE, target=''):
        def add(archive):
            item = tarfile.TarInfo(name)
            item.type = kind
            item.linkname = target
            archive.addfile(item)
        with self.archive(add) as archive, self.assertRaises(InputError):
            archive_members(archive, 'mesa-pin')

    def test_traversal(self):
        self.reject('mesa-pin/../escape')

    def test_empty_path(self):
        self.reject('.')

    def test_absolute_member(self):
        self.reject('/absolute')

    def test_duplicate_member(self):
        self.reject('mesa-pin/regular')

    def test_special_node(self):
        self.reject('mesa-pin/device', tarfile.CHRTYPE)

    def test_file_as_parent(self):
        self.reject('mesa-pin/regular/child')

    def test_escaping_link(self):
        self.reject('mesa-pin/link', tarfile.SYMTYPE, '../../escape')

    def test_cyclic_link(self):
        self.reject('mesa-pin/link', tarfile.SYMTYPE, 'link')

    def test_valid_confined_link(self):
        def add(archive):
            item = tarfile.TarInfo('mesa-pin/link')
            item.type, item.linkname = tarfile.SYMTYPE, 'regular'
            archive.addfile(item)
        with self.archive(add) as archive:
            self.assertEqual(len(archive_members(archive, 'mesa-pin')), 3)

    def test_archive_hash_before_parse(self):
        path = self.root / 'fake.tar.gz'
        path.write_bytes(b'not-an-archive')
        with self.assertRaises(InputError):
            verify_archive(path, load_lock(HERE / 'lock.json'))

    def test_source_pin_after_valid_archive_hash(self):
        revision = '0' * 40
        root = 'mesa-' + revision
        path = self.root / 'source.tar'
        with tarfile.open(path, 'w') as archive:
            directory = tarfile.TarInfo(root)
            directory.type = tarfile.DIRTYPE
            archive.addfile(directory)
            for name, raw in [('VERSION', b'25.3.6'), ('driver.c', b'altered')]:
                item = tarfile.TarInfo(root + '/' + name)
                item.size = len(raw)
                archive.addfile(item, io.BytesIO(raw))
        lock = {'mesa': {'revision': revision, 'version': '25.3.6', 'bytes': path.stat().st_size,
                        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                        'source_files': {'driver.c': hashlib.sha256(b'original').hexdigest()}}}
        with self.assertRaisesRegex(InputError, 'F02 source pin mismatch'):
            verify_archive(path, lock)

    def extracted(self):
        with self.archive(lambda archive: None) as archive:
            archive.extractall(self.root, filter='data')
            path = self.root / 'captured.tar'
            with path.open('wb') as stream:
                archive.fileobj.seek(0)
                stream.write(archive.fileobj.read())
        return path

    def test_extracted_source_addition(self):
        path = self.extracted()
        (self.root / 'mesa-pin/extra').write_text('unexpected')
        with self.assertRaisesRegex(InputError, 'added or missing'):
            verify_extracted(path, self.root, {'mesa': {'revision': 'pin'}})

    def test_extracted_source_modification(self):
        path = self.extracted()
        (self.root / 'mesa-pin/regular').write_text('modified')
        with self.assertRaisesRegex(InputError, 'differs from archive'):
            verify_extracted(path, self.root, {'mesa': {'revision': 'pin'}})


if __name__ == '__main__':
    unittest.main()
