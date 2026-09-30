"""Private tmpfs transfer admits only deterministic modules and returned metadata."""

import io
from pathlib import Path
import tarfile
import tempfile
import unittest

from kernel_transport import metadata, module_tar


def tar(nodes):
    result = io.BytesIO()
    with tarfile.open(fileobj=result, mode="w:") as archive:
        for name, kind, payload in nodes:
            member = tarfile.TarInfo(name)
            member.type = kind
            if kind == tarfile.REGTYPE:
                member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))
            else:
                member.linkname = payload.decode()
                archive.addfile(member)
    return result.getvalue()


class TransportTests(unittest.TestCase):
    def test_transfer_is_deterministic_and_contains_no_kernel_or_private_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "usr/lib/modules/test/kernel"
            source.mkdir(parents=True)
            (source / "gpu.ko.xz").write_bytes(b"gpu")
            (root / "lib").mkdir()
            (root / "lib/modules").symlink_to("../usr/lib/modules")
            (root / "private").write_bytes(b"must not transfer")
            raw = module_tar(root, "test")
            self.assertEqual(raw, module_tar(root, "test"))
            with tarfile.open(fileobj=io.BytesIO(raw)) as archive:
                names = archive.getnames()
                self.assertNotIn("private", names)
                self.assertEqual(archive.extractfile("usr/lib/modules/test/kernel/gpu.ko.xz").read(), b"gpu")
                self.assertEqual(archive.getmember("lib/modules").linkname, "../usr/lib/modules")
            (source / "evil.ko").symlink_to("/private")
            with self.assertRaises(ValueError):
                module_tar(root, "test")

    def test_returned_metadata_is_atomic_on_validation_failure(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            directory = root / "usr/lib/modules/test"
            directory.mkdir(parents=True)
            (directory / "modules.dep").write_bytes(b"previous")
            prefix = "usr/lib/modules/test/"
            normal = [(prefix + name, tarfile.REGTYPE, b"new")
                      for name in ("modules.dep", "modules.dep.bin", "modules.softdep")]
            bad = [("../escape", tarfile.REGTYPE, b"x"), (prefix + "modules.dep", tarfile.REGTYPE, b"duplicate"),
                   (prefix + "modules.alias", tarfile.SYMTYPE, b"/escape"),
                   (prefix + "kernel/gpu.ko.xz", tarfile.REGTYPE, b"x")]
            for member in bad:
                with self.subTest(member=member), self.assertRaises(ValueError):
                    metadata(tar(normal + [member]), root, "test")
                self.assertEqual((directory / "modules.dep").read_bytes(), b"previous")
            with self.assertRaises(ValueError):
                metadata(tar(normal[:1]), root, "test")
            metadata(tar(normal), root, "test")
            self.assertEqual((directory / "modules.dep").read_bytes(), b"new")


if __name__ == "__main__":
    unittest.main()
