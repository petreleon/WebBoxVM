"""Keep absolute guest links and executable bits intact on host export."""
import os
import pathlib
import tempfile
import unittest

from export import export_tree, preserve_install_modes
from paths import validate_links


class ExportTests(unittest.TestCase):
    def test_absolute_guest_link_and_executable_mode(self):
        with tempfile.TemporaryDirectory() as temp:
            base = pathlib.Path(temp)
            source, target = base / 'source', base / 'target'
            (source / 'usr/bin').mkdir(parents=True)
            (source / 'bin').mkdir()
            program = source / 'usr/bin/guest-program'
            program.write_bytes(b'executable')
            program.chmod(0o755)
            (source / 'bin/run').symlink_to('/usr/bin/guest-program')
            export_tree(source, target)
            self.assertEqual(os.readlink(target / 'bin/run'), '/usr/bin/guest-program')
            self.assertEqual((target / 'usr/bin/guest-program').stat().st_mode & 0o777, 0o755)
            validate_links(target)

    def test_existing_export_is_not_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            base = pathlib.Path(temp)
            (base / 'source').mkdir()
            (base / 'target').mkdir()
            with self.assertRaises(FileExistsError):
                export_tree(base / 'source', base / 'target')

    def test_recovery_restores_install_modes_and_rejects_byte_drift(self):
        with tempfile.TemporaryDirectory() as temp:
            base = pathlib.Path(temp)
            source, target = base / 'source', base / 'target'
            source.mkdir()
            file = source / 'library.so'
            file.write_bytes(b'compiled library')
            file.chmod(0o755)
            export_tree(source, target)
            (target / file.name).chmod(0o644)
            preserve_install_modes(source, target)
            self.assertEqual((target / file.name).stat().st_mode & 0o777, 0o755)
            (target / file.name).write_bytes(b'drift')
            with self.assertRaisesRegex(ValueError, 'bytes differ'):
                preserve_install_modes(source, target)


if __name__ == '__main__':
    unittest.main()
