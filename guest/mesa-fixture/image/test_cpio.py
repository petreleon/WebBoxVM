"""Decode Linux newc independently and exercise bounded archive publication."""

import hashlib
import io
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

import cpio


def decode(raw):
    position, entries = 0, []
    while position < len(raw):
        header = raw[position:position + 110]
        if header[:6] != b"070701" or len(header) != 110:
            raise AssertionError("invalid newc header")
        values = [int(header[start:start + 8], 16) for start in range(6, 110, 8)]
        position += 110
        name = raw[position:position + values[11]]
        if not name.endswith(b"\0"):
            raise AssertionError("missing name terminator")
        position = (position + values[11] + 3) & ~3
        data = raw[position:position + values[6]]
        position = (position + values[6] + 3) & ~3
        entries.append((name[:-1].decode(), values, data))
        if name == b"TRAILER!!!\0":
            if position != len(raw):
                raise AssertionError("trailing archive data")
            return entries
    raise AssertionError("archive has no trailer")


class CpioTests(unittest.TestCase):
    def test_independent_headers_payloads_and_device_numbers(self):
        nodes = {"file": {"mode": stat.S_IFREG | 0o755, "data": b"abcde"},
                 "dir": {"mode": stat.S_IFDIR | 0o755, "data": b""},
                 "link": {"mode": stat.S_IFLNK | 0o777, "data": b"file"},
                 "console": {"mode": stat.S_IFCHR | 0o600, "data": b"", "major": 5, "minor": 1}}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "archive"
            result = cpio.write(path, nodes)
            raw = path.read_bytes()
        entries = decode(raw)
        self.assertEqual([entry[0] for entry in entries], ["console", "dir", "file", "link", "TRAILER!!!"])
        self.assertEqual(entries[2][2], b"abcde")
        self.assertEqual(entries[3][2], b"file")
        self.assertEqual(entries[0][1][9:11], [5, 1])
        self.assertEqual(entries[1][1][4], 2)
        self.assertEqual(entries[2][1][1], stat.S_IFREG | 0o755)
        self.assertTrue(all(item[1][2:4] == [0, 0] and item[1][5] == 0 for item in entries))
        self.assertEqual(result, {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})

    def test_deterministic_sorted_members(self):
        nodes = {name: {"mode": stat.S_IFREG | 0o644, "data": name.encode()} for name in ("z", "a")}
        with tempfile.TemporaryDirectory() as directory:
            first, second = Path(directory) / "a", Path(directory) / "b"
            self.assertEqual(cpio.write(first, nodes), cpio.write(second, dict(reversed(list(nodes.items())))))
            self.assertEqual(first.read_bytes(), second.read_bytes())

    def test_rejects_unsafe_and_noncanonical_names(self):
        for name in ("", "/a", "../a", "a/../b", "a\0b", "a//b", "a/./b", "a/"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                cpio.entry(io.BytesIO(), 1, name, stat.S_IFREG, b"")

    def test_strict_limit_and_uint32_fields(self):
        output = io.BytesIO()
        cpio.entry(output, 1, "a", stat.S_IFREG, b"hello")
        size = output.tell()
        with patch.object(cpio, "MAX_INITRD", size), self.assertRaises(ValueError):
            cpio.entry(io.BytesIO(), 1, "a", stat.S_IFREG, b"hello")
        with patch.object(cpio, "MAX_INITRD", size + 1):
            cpio.entry(io.BytesIO(), 1, "a", stat.S_IFREG, b"hello")
        for number in (-1, 0x100000000):
            with self.subTest(number=number), self.assertRaises(ValueError):
                cpio.entry(io.BytesIO(), number, "a", stat.S_IFREG, b"")


if __name__ == "__main__":
    unittest.main()
