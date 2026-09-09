"""No-follow descriptor traversal for the isolated successor cache."""

from __future__ import annotations

import os
from pathlib import Path, PurePosixPath

from successor_cache_model import reject

NOFOLLOW = getattr(os, "O_NOFOLLOW", 0)
if not NOFOLLOW:
    raise RuntimeError("successor cache needs O_NOFOLLOW")
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)


def relative(value: str) -> tuple[str, ...]:
    if not isinstance(value, str) or not value or "\x00" in value:
        reject("successor cache path is unsafe")
    path = PurePosixPath(value)
    if (
        not path.parts
        or path.is_absolute()
        or str(path) != value
        or any(part in (".", "..") for part in path.parts)
    ):
        reject("successor cache path is unsafe")
    return path.parts


def close(descriptor: int) -> None:
    try:
        os.close(descriptor)
    except OSError:
        pass


def child(parent: int, name: str, create: bool) -> int:
    try:
        return os.open(name, DIR_FLAGS, dir_fd=parent)
    except FileNotFoundError:
        if not create:
            raise
        try:
            os.mkdir(name, 0o700, dir_fd=parent)
        except FileExistsError:
            pass
        except (OSError, ValueError) as error:
            reject(f"successor cache directory is a symlink or unsafe: {error}")
        try:
            return os.open(name, DIR_FLAGS, dir_fd=parent)
        except (OSError, ValueError) as error:
            reject(f"successor cache directory is a symlink or unsafe: {error}")
    except (OSError, ValueError) as error:
        reject(f"successor cache directory is a symlink or unsafe: {error}")


def directory_fd(root: Path, create: bool) -> int:
    if not isinstance(root, Path) or not root.is_absolute():
        reject("successor cache root must be an explicit absolute path")
    try:
        descriptor = os.open("/", DIR_FLAGS)
    except (OSError, ValueError) as error:
        reject(f"successor cache root is unsafe: {error}")
    try:
        for part in root.parts[1:]:
            next_descriptor = child(descriptor, part, create)
            close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except BaseException:
        close(descriptor)
        raise


def parent_fd(root: Path, value: str, create: bool) -> tuple[int, str]:
    parts = relative(value)
    descriptor = directory_fd(root, create)
    try:
        for part in parts[:-1]:
            next_descriptor = child(descriptor, part, create)
            close(descriptor)
            descriptor = next_descriptor
        return descriptor, parts[-1]
    except BaseException:
        close(descriptor)
        raise
