"""Regressions for guest-root absolute, relative and merged-/usr links."""
import pathlib
import tempfile
import unittest

from paths import rooted_path, validate_links


class RuntimePathTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / 'usr/lib').mkdir(parents=True)
        (self.root / 'usr/lib/loader').write_bytes(b'guest-loader')

    def test_absolute_link_uses_guest_bytes(self):
        (self.root / 'loader').symlink_to('/usr/lib/loader')
        self.assertEqual(rooted_path(self.root, 'loader').read_bytes(), b'guest-loader')
        validate_links(self.root)

    def test_merged_usr_ancestor(self):
        (self.root / 'lib').symlink_to('usr/lib')
        (self.root / 'usr/lib/alias').symlink_to('loader')
        self.assertEqual(rooted_path(self.root, 'lib/alias').read_bytes(), b'guest-loader')
        validate_links(self.root)

    def test_missing_logical_loader_target(self):
        (self.root / 'lib').mkdir()
        (self.root / 'lib/ld-linux-aarch64.so.1').symlink_to('aarch64-linux-gnu/ld-linux-aarch64.so.1')
        with self.assertRaisesRegex(ValueError, 'missing guest runtime'):
            validate_links(self.root)

    def test_escape(self):
        (self.root / 'link').symlink_to('../escape')
        with self.assertRaisesRegex(ValueError, 'escapes'):
            rooted_path(self.root, 'link')

    def test_cycle(self):
        (self.root / 'a').symlink_to('b')
        (self.root / 'b').symlink_to('a')
        with self.assertRaisesRegex(ValueError, 'cyclic'):
            rooted_path(self.root, 'a')


if __name__ == '__main__':
    unittest.main()
