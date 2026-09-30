"""Archive confinement and canonical package/kernel admission regressions."""

import io
import lzma
from pathlib import Path
import tarfile
import tempfile
import unittest

import kernel
import kernel_archive as archive


def tar(nodes):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:") as stream:
        for name, kind, data in nodes:
            member = tarfile.TarInfo(name)
            member.type = kind
            if kind == tarfile.REGTYPE:
                member.size = len(data)
                stream.addfile(member, io.BytesIO(data))
            else:
                member.linkname = data.decode()
                stream.addfile(member)
    return lzma.compress(output.getvalue())


def deb(members):
    output = bytearray(b"!<arch>\n")
    for name, raw in members:
        header = f"{name + '/':<16}{0:<12}{0:<6}{0:<6}{'100644':<8}{len(raw):<10}`\n".encode()
        output.extend(header + raw + (b"\n" if len(raw) % 2 else b""))
    return bytes(output)


class ArchiveTests(unittest.TestCase):
    def test_debian_format_rejects_extra_duplicate_truncated_and_malformed_members(self):
        valid = [("debian-binary", b"2.0\n"), ("control.tar.xz", b"c"), ("data.tar.xz", b"d")]
        self.assertEqual(set(archive.ar_members(deb(valid))), {name for name, _ in valid})
        for bad in [deb(valid + [("data.tar.xz", b"x")]), deb(valid + [("postinst", b"exit 0")]),
                    deb(valid)[:-1], b"!<arch>\ninvalid", deb([("debian-binary", b"1.0\n"), *valid[1:]])]:
            with self.subTest(bad=bad[:32]), self.assertRaises(ValueError):
                archive.ar_members(bad)

    def test_tar_rejects_escaping_links_devices_duplicate_aliases_and_file_ancestors(self):
        for nodes in [[("../escape", tarfile.REGTYPE, b"x")], [("/absolute", tarfile.REGTYPE, b"x")],
                      [("bad\\name", tarfile.REGTYPE, b"x")], [("link", tarfile.SYMTYPE, b"../escape")],
                      [("node", tarfile.CHRTYPE, b"")], [("a", tarfile.REGTYPE, b""), ("./a", tarfile.REGTYPE, b"")],
                      [("a", tarfile.REGTYPE, b""), ("a/b", tarfile.REGTYPE, b"x")],
                      [("missing/a", tarfile.REGTYPE, b"x")]]:
            with self.subTest(nodes=nodes), self.assertRaises(ValueError):
                archive.open_tar(tar(nodes))

    def test_archive_expansion_is_bounded(self):
        with self.assertRaises(ValueError):
            archive.open_tar(tar([("large", tarfile.REGTYPE, b"x" * 4096)]), limit=100)

    def test_control_identity_is_checked_without_extracting_or_executing_postinst(self):
        control = b"Package: linux-image-test\nVersion: 1\nArchitecture: arm64\n"
        raw = tar([("control", tarfile.REGTYPE, control), ("postinst", tarfile.REGTYPE, b"exit 99")])
        archive.package_identity(raw, "linux-image-test", "1")
        for package, version in [("wrong", "1"), ("linux-image-test", "2")]:
            with self.assertRaises(ValueError):
                archive.package_identity(raw, package, version)
        wrong = tar([("control", tarfile.REGTYPE, control.replace(b"arm64", b"amd64"))])
        with self.assertRaises(ValueError):
            archive.package_identity(wrong, "linux-image-test", "1")

    def test_extraction_writes_only_matching_kernel_and_modules(self):
        release = "test"
        dirs = ["boot", "usr", "usr/lib", "usr/lib/modules", "usr/lib/modules/test"]
        nodes = [(name, tarfile.DIRTYPE, b"") for name in dirs]
        nodes += [("boot/vmlinuz-test", tarfile.REGTYPE, b"Image"),
                  ("usr/lib/modules/test/modules.builtin", tarfile.REGTYPE, b"builtin\n"),
                  ("boot/vmlinuz-other", tarfile.REGTYPE, b"wrong"),
                  ("postinst", tarfile.REGTYPE, b"must not execute")]
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            rows = archive.extract(tar(nodes), root, release)
            self.assertEqual(set(rows), {"boot/vmlinuz-test", "usr/lib/modules/test/modules.builtin"})
            self.assertEqual((root / "boot/vmlinuz-test").read_bytes(), b"Image")
            self.assertFalse((root / "postinst").exists())
            self.assertFalse((root / "boot/vmlinuz-other").exists())

    def test_wrong_package_pin_cannot_publish_or_invoke_tools(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "bad.deb").write_bytes(b"wrong")
            with self.assertRaises(ValueError):
                kernel.build(root / "bad.deb", root / "output", root / "logs")
            self.assertFalse((root / "output").exists())
            self.assertFalse((root / "logs").exists())

    def test_output_and_log_parent_symlinks_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "alias").symlink_to(root, target_is_directory=True)
            with self.assertRaises(ValueError):
                kernel.confined_parent(root / "alias/new")
            self.assertFalse((root / "new").exists())


if __name__ == "__main__":
    unittest.main()
