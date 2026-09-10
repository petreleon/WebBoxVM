"""No-follow descriptor traversal and external-root validation for Docs staging."""

from __future__ import annotations

import os
from pathlib import Path, PurePosixPath

from vulkan_docs_stage_model import reject

NOFOLLOW = getattr(os, "O_NOFOLLOW", 0)
if not NOFOLLOW:
    raise RuntimeError("Docs staging requires O_NOFOLLOW")
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)


def relative(value: str) -> tuple[str, ...]:
    if not isinstance(value, str) or not value or "\x00" in value or "\\" in value or any(char in value for char in "?#%"):
        reject("Docs staging path is unsafe")
    path = PurePosixPath(value)
    if not path.parts or path.is_absolute() or str(path) != value or any(part in (".", "..") for part in path.parts):
        reject("Docs staging path is unsafe")
    return path.parts


def close(descriptor: int) -> None:
    try:
        os.close(descriptor)
    except OSError:
        pass


def _private(descriptor: int, label: str) -> None:
    try:
        info = os.fstat(descriptor)
    except OSError as error:
        reject(f"Docs staging {label} cannot be inspected safely: {error}")
    if info.st_uid != os.geteuid() or info.st_mode & 0o022:
        reject(f"Docs staging {label} is not private to the current user")


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
            reject(f"Docs staging directory is unsafe: {error}")
        try:
            return os.open(name, DIR_FLAGS, dir_fd=parent)
        except (OSError, ValueError) as error:
            reject(f"Docs staging directory is unsafe: {error}")
    except (OSError, ValueError) as error:
        reject(f"Docs staging directory is unsafe: {error}")


def _directory_fd(root: Path, create: bool) -> int:
    if not isinstance(root, Path) or not root.is_absolute():
        reject("Docs staging root must be an explicit absolute path")
    try:
        descriptor = os.open("/", DIR_FLAGS)
    except (OSError, ValueError) as error:
        reject(f"Docs staging root is unsafe: {error}")
    try:
        for part in root.parts[1:]:
            next_descriptor = child(descriptor, part, create)
            close(descriptor)
            descriptor = next_descriptor
        _private(descriptor, "root")
        return descriptor
    except BaseException:
        close(descriptor)
        raise


def _parent_fd(root: Path, value: str, create: bool) -> tuple[int, str]:
    parts = relative(value)
    descriptor = _directory_fd(root, create)
    try:
        for part in parts[:-1]:
            next_descriptor = child(descriptor, part, create)
            try:
                _private(next_descriptor, "directory")
            except BaseException:
                close(next_descriptor)
                raise
            close(descriptor)
            descriptor = next_descriptor
        return descriptor, parts[-1]
    except BaseException:
        close(descriptor)
        raise


def external_root(value: Path, repository: Path) -> Path:
    if not isinstance(value, Path) or not isinstance(repository, Path) or not value.is_absolute() or value.anchor != "/":
        reject("Docs staging root must be an explicit absolute path")
    if "\x00" in str(value) or any(part in (".", "..") for part in value.parts):
        reject("Docs staging root has an unsafe path component")
    root = Path(value.anchor)
    for part in value.parts[1:]:
        root /= part
        try:
            if root.is_symlink():
                reject("Docs staging root has a symlink component")
        except (OSError, ValueError, RuntimeError) as error:
            reject(f"Docs staging root cannot be inspected safely: {error}")
    if root == Path(root.anchor):
        reject("Docs staging root cannot be a filesystem root")
    try:
        repo = repository.resolve(strict=True)
        if repository.is_symlink() or not repo.is_dir():
            reject("Docs staging repository anchor is unsafe")
        if root == repo or root.is_relative_to(repo) or repo.is_relative_to(root):
            reject("Docs staging root must be outside the repository")
        for candidate in (root, *root.parents):
            if candidate.exists() and candidate.samefile(repo):
                reject("Docs staging root must be outside the repository")
        if root.exists() and not root.is_dir():
            reject("Docs staging root is not a directory")
    except (OSError, ValueError, RuntimeError) as error:
        reject(f"Docs staging root cannot be inspected safely: {error}")
    return root
