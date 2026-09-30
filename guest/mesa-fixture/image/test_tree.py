"""Guest symlinks must resolve within a complete, physically confined tree."""

import os
from pathlib import Path
import stat
import tempfile
import unittest

import tree


class TreeTests(unittest.TestCase):
    def nodes(self):
        nodes = {}
        tree.add(nodes, "usr", stat.S_IFDIR | 0o755)
        tree.add(nodes, "usr/bin", stat.S_IFDIR | 0o755)
        tree.add(nodes, "usr/bin/sh", stat.S_IFREG | 0o755, b"shell")
        tree.add(nodes, "bin", stat.S_IFLNK | 0o777, b"usr/bin")
        tree.add(nodes, "shell", stat.S_IFLNK | 0o777, b"/bin/sh")
        return nodes

    def test_merged_usr_and_absolute_guest_links(self):
        nodes = self.nodes()
        tree.validate(nodes)
        self.assertEqual(tree.resolve(nodes, "bin/sh"), "usr/bin/sh")
        self.assertEqual(tree.resolve(nodes, "shell"), "usr/bin/sh")

    def test_rejects_missing_cycle_escape_and_empty_links(self):
        for target in (b"missing", b"self", b"../outside", b"", b"usr/bin/sh/child", b"usr\0/bin"):
            nodes = self.nodes()
            tree.add(nodes, "self", stat.S_IFLNK | 0o777, target)
            with self.subTest(target=target), self.assertRaises(ValueError):
                tree.validate(nodes)

    def test_rejects_physical_child_below_symlink(self):
        nodes = self.nodes()
        tree.add(nodes, "bin/evil", stat.S_IFREG | 0o644, b"payload")
        with self.assertRaisesRegex(ValueError, "unsafe ancestor"):
            tree.validate(nodes)

    def test_conflicting_inputs_cannot_overwrite(self):
        nodes = self.nodes()
        tree.add(nodes, "usr/bin/sh", stat.S_IFREG | 0o755, b"shell")
        with self.assertRaisesRegex(ValueError, "conflict"):
            tree.add(nodes, "usr/bin/sh", stat.S_IFREG | 0o755, b"replacement")

    def test_canonical_names_only(self):
        for name in ("", "/a", "../a", "a/../b", "a//b", "a/./b", "a/", "a\0b"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                tree.add({}, name, stat.S_IFDIR)

    def test_collection_preserves_links_permissions_not_inode_or_mtime(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "usr").mkdir()
            (root / "usr/sh").write_bytes(b"shell")
            (root / "usr/sh").chmod(0o755)
            (root / "shell").symlink_to("usr/sh")
            first = tree.collect(root)
            os.utime(root / "usr/sh", (42, 42))
            self.assertEqual(first, tree.collect(root))
            self.assertEqual(first["shell"]["data"], b"usr/sh")
            self.assertEqual(first["usr/sh"]["mode"], stat.S_IFREG | 0o755)

    def test_rejects_special_input_and_symlink_root(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            os.mkfifo(root / "pipe")
            with self.assertRaisesRegex(ValueError, "unsupported"):
                tree.collect(root)
            (root / "alias").symlink_to(root, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "regular directory"):
                tree.collect(root / "alias")


if __name__ == "__main__":
    unittest.main()
