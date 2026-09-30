"""Retain dlopen libraries omitted by static tool ELF dependencies."""
from pathlib import Path
import unittest
from unittest.mock import patch

import runtime


class RuntimeLibraryTests(unittest.TestCase):
    def test_vulkan_loader_is_an_independent_closure_root(self):
        path = Path('/usr/lib/aarch64-linux-gnu/libvulkan.so.1')
        queue = []
        with patch.object(runtime, 'copy', return_value=path) as copy:
            runtime.seed_libraries({'libvulkan.so.1': path}, queue)
        copy.assert_called_once_with(path)
        self.assertEqual(queue, [path.resolve()])

    def test_missing_loader_fails_before_publishing_runtime(self):
        with self.assertRaisesRegex(ValueError, 'libvulkan.so.1'):
            runtime.seed_libraries({}, [])


if __name__ == '__main__':
    unittest.main()
