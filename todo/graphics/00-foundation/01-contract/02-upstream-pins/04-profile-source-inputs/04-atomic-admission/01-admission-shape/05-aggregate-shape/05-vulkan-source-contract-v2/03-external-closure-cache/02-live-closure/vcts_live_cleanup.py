"""Descriptor-anchored removal for a failed or consumed live VCTS stage."""

from __future__ import annotations

import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE.parent / "01-cache-contract"
if str(CACHE) not in sys.path:
    sys.path.insert(0, str(CACHE))
import vcts_cache_fs as fs


def discard_payloads(directory_fd: int, parent_fd: int, leaf: str, names: tuple[str, ...]) -> None:
    """Remove only known names under a stage directory already opened by descriptor."""
    try:
        fs.name(leaf)
        for name in names:
            fs.name(name)
            try:
                os.unlink(name, dir_fd=directory_fd)
            except FileNotFoundError:
                pass
        os.fsync(directory_fd)
        try:
            os.close(directory_fd)
        finally:
            directory_fd = -1
        os.rmdir(leaf, dir_fd=parent_fd)
        os.fsync(parent_fd)
    except OSError as error:
        fs.reject(f"live stage cannot be safely discarded: {error.strerror}")
    finally:
        if directory_fd >= 0:
            try:
                os.close(directory_fd)
            except OSError:
                pass
        if parent_fd >= 0:
            try:
                os.close(parent_fd)
            except OSError:
                pass
