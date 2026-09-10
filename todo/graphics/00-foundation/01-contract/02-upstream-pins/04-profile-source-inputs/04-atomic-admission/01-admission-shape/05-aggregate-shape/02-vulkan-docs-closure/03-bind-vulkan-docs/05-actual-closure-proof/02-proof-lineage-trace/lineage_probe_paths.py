"""Descriptor-checked source and artifact roots for the ptrace gate."""

from __future__ import annotations

import hashlib
import os
import re
import stat
from dataclasses import dataclass
from pathlib import Path

from lineage_model import reject

HERE = Path(__file__).resolve().parent
TOKEN = re.compile(r"^[a-z0-9][a-z0-9-]{0,63}$")
WORK_PREFIX = "f02.4.4.1.5.2.3.5.2-ptrace-probe-"
SOURCE = HERE / "observer/lineage_ptrace_probe.c"
SOURCE_SHA256 = "35f7f8f3bedb243be57769c788f91406fc5b4127aded39fd162516892088144f"


def _repository() -> Path:
    for candidate in (HERE, *HERE.parents):
        if (candidate / ".git").exists(): return candidate.resolve(strict=True)
    reject("ptrace gate cannot locate its repository root")


REPOSITORY = _repository()
WORK_PARENT = REPOSITORY / ".artifacts/graphics"


@dataclass
class WorkRoot:
    path: Path
    parent_fd: int
    parent_identity: tuple[int, int]
    child_identity: tuple[int, int]

    def close(self) -> None: os.close(self.parent_fd)


def _relative(root: Path, target: Path, label: str) -> tuple[str, ...]:
    try: result = target.relative_to(root).parts
    except ValueError: reject(f"{label} escapes the repository")
    if not result: reject(f"{label} has no path components")
    return result


def safe_dir(root: Path, target: Path, label: str) -> Path:
    if root.is_symlink() or not root.is_dir() or not target.is_absolute(): reject(f"{label} has an unsafe root")
    current = root
    for component in _relative(root, target, label):
        current /= component
        if current.is_symlink() or not current.is_dir(): reject(f"{label} has a symlink or non-directory ancestor")
    return current


def safe_file(root: Path, target: Path, label: str) -> Path:
    safe_dir(root, target.parent, label)
    if target.is_symlink() or not target.is_file(): reject(f"{label} is not a regular local file")
    return target


def _identity(value: os.stat_result) -> tuple[int, int]: return value.st_dev, value.st_ino


def source_file() -> tuple[Path, tuple[int, int]]:
    source = safe_file(REPOSITORY, SOURCE, "ptrace gate source")
    try: payload, state = source.read_bytes(), source.stat()
    except OSError as error: reject(f"ptrace gate source cannot be read: {error}")
    if hashlib.sha256(payload).hexdigest() != SOURCE_SHA256: reject("ptrace gate source digest is not reviewed")
    return source, _identity(state)


def same_source(expected: tuple[Path, tuple[int, int]]) -> None:
    if source_file() != expected: reject("ptrace gate source changed during the capability probe")


def work_path(work: Path) -> Path:
    if not work.is_absolute() or work.parent != WORK_PARENT or not work.name.startswith(WORK_PREFIX):
        reject("ptrace gate work root is outside its dedicated artifact parent")
    if not TOKEN.fullmatch(work.name.removeprefix(WORK_PREFIX)): reject("ptrace gate work root has an unsafe token")
    safe_dir(REPOSITORY, WORK_PARENT, "ptrace gate artifact parent")
    return work


def _open_parent() -> tuple[int, tuple[int, int]]:
    safe_dir(REPOSITORY, WORK_PARENT, "ptrace gate artifact parent")
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0)
    try: current = os.open(str(REPOSITORY), flags)
    except OSError as error: reject(f"ptrace gate artifact root cannot be opened: {error}")
    try:
        for component in _relative(REPOSITORY, WORK_PARENT, "ptrace gate artifact parent"):
            successor = os.open(component, flags, dir_fd=current)
            os.close(current); current = successor
        return current, _identity(os.fstat(current))
    except OSError as error:
        os.close(current); reject(f"ptrace gate artifact parent cannot be opened: {error}")


def work_root(work: Path) -> WorkRoot:
    work = work_path(work); parent_fd, parent_identity = _open_parent()
    try:
        os.mkdir(work.name, mode=0o700, dir_fd=parent_fd)
        child = os.stat(work.name, dir_fd=parent_fd, follow_symlinks=False)
    except (FileExistsError, OSError) as error:
        os.close(parent_fd); reject(f"ptrace gate work root must be absent and safe: {error}")
    if not stat.S_ISDIR(child.st_mode):
        os.close(parent_fd); reject("ptrace gate work root is not a directory")
    return WorkRoot(work, parent_fd, parent_identity, _identity(child))


def recheck(work: WorkRoot) -> None:
    try: child = os.stat(work.path.name, dir_fd=work.parent_fd, follow_symlinks=False)
    except OSError as error: reject(f"ptrace gate work root changed: {error}")
    live_fd, live_identity = _open_parent()
    os.close(live_fd)
    if (_identity(os.fstat(work.parent_fd)) != work.parent_identity or live_identity != work.parent_identity
            or not stat.S_ISDIR(child.st_mode) or _identity(child) != work.child_identity):
        reject("ptrace gate work root or ancestor changed")
