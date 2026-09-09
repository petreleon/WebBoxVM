"""Descriptor-race regressions for isolated successor-cache filesystem writes."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

import successor_cache_fs as FS
import successor_cache_lock as LOCK
import successor_cache_paths as PATHS
from successor_cache_model import CacheError
from successor_cache_paths import relative


class HostileFilesystemTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = (Path(self.temporary.name) / "external").resolve(strict=False)

    def test_parent_swap_cannot_redirect_an_atomic_member_write(self) -> None:
        outside = Path(self.temporary.name) / "outside"
        outside.mkdir()
        original = FS.os.link

        def swap_parent(*args, **kwargs):
            result = original(*args, **kwargs)
            parent = self.root / "safe"
            parent.rename(self.root / "moved-safe")
            parent.symlink_to(outside, target_is_directory=True)
            return result

        with mock.patch.object(FS.os, "link", side_effect=swap_parent):
            with self.assertRaisesRegex(CacheError, "symlink or unsafe|parent changed"):
                FS.atomic_file(self.root, "safe/member", b"payload", "test member")
        self.assertFalse((outside / "member").exists())
        self.assertEqual((self.root / "moved-safe" / "member").read_bytes(), b"payload")

    def test_lock_guard_rejects_unlink_and_recreate_of_its_lock_inode(self) -> None:
        lock_path = self.root / "locks" / "closure.lock"
        with self.assertRaisesRegex(CacheError, "lock changed"):
            with LOCK.cache_lock(self.root, "locks/closure.lock", exclusive=True, create=True) as guard:
                lock_path.rename(self.root / "locks" / "old.lock")
                lock_path.write_bytes(b"replacement")
                guard()

    def test_lock_acquisition_os_errors_do_not_escape_the_cache_contract(self) -> None:
        with mock.patch.object(LOCK.fcntl, "flock", side_effect=OSError("unavailable")):
            with self.assertRaisesRegex(CacheError, "cannot be acquired safely"):
                with LOCK.cache_lock(self.root, "locks/closure.lock", exclusive=True, create=True):
                    pass

    def test_read_cap_rejects_a_member_that_grows_after_its_stat(self) -> None:
        FS.atomic_file(self.root, "safe/member", b"x", "test member")
        path = self.root / "safe" / "member"
        original, appended = FS.os.read, False

        def grow_after_first_read(descriptor: int, count: int) -> bytes:
            nonlocal appended
            payload = original(descriptor, count)
            if not appended:
                appended = True
                with path.open("ab") as output:
                    output.write(b"z" * (2 * 1024 * 1024))
            return payload

        with mock.patch.object(FS.os, "read", side_effect=grow_after_first_read):
            with self.assertRaisesRegex(CacheError, "byte count mismatch"):
                FS.read_file(self.root, "safe/member", "test member", expected_bytes=1)

    def test_dot_path_is_rejected_before_descriptor_traversal(self) -> None:
        with self.assertRaisesRegex(CacheError, "path is unsafe"):
            relative(".")
        with self.assertRaisesRegex(CacheError, "path is unsafe"):
            FS.atomic_file(self.root, ".", b"payload", "test member")

    def test_descriptor_root_and_mkdir_errors_stay_inside_the_contract(self) -> None:
        with mock.patch.object(PATHS.os, "open", side_effect=OSError("denied")):
            with self.assertRaisesRegex(CacheError, "root is unsafe"):
                FS.atomic_file(self.root, "safe/member", b"payload", "test member")
        with mock.patch.object(PATHS.os, "mkdir", side_effect=PermissionError("denied")):
            with self.assertRaisesRegex(CacheError, "directory is a symlink or unsafe"):
                FS.atomic_file(self.root, "safe/member", b"payload", "test member")

    def test_atomic_unlink_and_stat_errors_stay_inside_the_contract(self) -> None:
        with mock.patch.object(FS.os, "unlink", side_effect=OSError("denied")):
            with self.assertRaisesRegex(CacheError, "cannot finalize staging safely"):
                FS.atomic_file(self.root, "safe/member", b"payload", "test member")
        with mock.patch.object(FS.os, "fstat", side_effect=OSError("denied")):
            with self.assertRaisesRegex(CacheError, "cannot be inspected safely"):
                FS.atomic_file(self.root, "other/member", b"payload", "test member")

    def test_unlock_failure_still_closes_the_lock_and_parent_descriptors(self) -> None:
        original_flock, original_close, closed = LOCK.fcntl.flock, LOCK.close, []

        def unlock_fails(descriptor: int, operation: int) -> None:
            if operation == LOCK.fcntl.LOCK_UN:
                raise OSError("denied")
            original_flock(descriptor, operation)

        def record_close(descriptor: int) -> None:
            closed.append(descriptor)
            original_close(descriptor)

        with mock.patch.object(LOCK.fcntl, "flock", side_effect=unlock_fails):
            with mock.patch.object(LOCK, "close", side_effect=record_close):
                with self.assertRaisesRegex(CacheError, "cannot be released safely"):
                    with LOCK.cache_lock(self.root, "locks/closure.lock", exclusive=True, create=True):
                        pass
        self.assertEqual(len(closed), 2)


if __name__ == "__main__":
    unittest.main()
