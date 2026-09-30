"""Prevent stale pre-Mesa25 alias assumptions or fallback ICDs in the fixture."""
import pathlib
import tempfile
import unittest

from layout import driver_layout


class LayoutTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / 'lib').mkdir()
        (self.root / 'share/vulkan/icd.d').mkdir(parents=True)
        (self.root / 'lib/libgallium-25.3.6.so').touch()
        (self.root / 'share/vulkan/icd.d/virtio_icd.aarch64.json').write_text('{}')

    def test_stock_versioned_library_without_legacy_alias(self):
        value = driver_layout(self.root, '25.3.6')
        self.assertEqual(value['drivers'], ['virgl'])
        self.assertEqual(value['legacy_dri_aliases'], [])

    def test_legacy_alias_rejected(self):
        (self.root / 'lib/virtio_gpu_dri.so').touch()
        with self.assertRaises(ValueError):
            driver_layout(self.root, '25.3.6')

    def test_extra_icd_rejected(self):
        (self.root / 'share/vulkan/icd.d/lvp_icd.aarch64.json').write_text('{}')
        with self.assertRaises(ValueError):
            driver_layout(self.root, '25.3.6')

    def test_wrong_version_rejected(self):
        with self.assertRaises(ValueError):
            driver_layout(self.root, '25.3.5')


if __name__ == '__main__':
    unittest.main()
