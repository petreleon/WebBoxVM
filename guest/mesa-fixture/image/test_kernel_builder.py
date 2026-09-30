"""Fresh locally built IDs remain sealed to the pinned recipe's provenance."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import kernel_tools as tools


class BuilderTests(unittest.TestCase):
    def test_fresh_immutable_id_accepts_matching_base_snapshot_and_architecture(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "builder.json"
            lock = json.loads((tools.RECIPE / "build/lock.json").read_text())
            identifier = "sha256:" + "1" * 64
            record = {"image_id": identifier, "native_architecture": "arm64",
                      "base_image": lock["base_image"], "snapshot": lock["snapshot"]}
            path.write_text(json.dumps(record))
            with patch.object(tools, "BUILDER_MANIFEST", path):
                self.assertEqual(tools.authorized_builder(identifier), record)
                for key, bad in (("image_id", "sha256:" + "2" * 64), ("native_architecture", "amd64"),
                                 ("base_image", "debian:latest"), ("snapshot", "latest")):
                    changed = {**record, key: bad}
                    path.write_text(json.dumps(changed))
                    with self.subTest(key=key), self.assertRaises(ValueError):
                        tools.authorized_builder(identifier)
                with self.assertRaises(ValueError):
                    tools.authorized_builder("debian:latest")

    def test_redirected_provenance_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "actual").write_text("{}")
            (root / "builder.json").symlink_to(root / "actual")
            with patch.object(tools, "BUILDER_MANIFEST", root / "builder.json"), self.assertRaises(ValueError):
                tools.authorized_builder(tools.BUILDER)


if __name__ == "__main__":
    unittest.main()
