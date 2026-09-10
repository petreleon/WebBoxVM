"""Read the reviewed Vulkan Docs tree solely through immutable Git blobs."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Callable

HERE = Path(__file__).resolve().parent
BASE = HERE.parent.parent
for directory in (BASE / "02-actual-closure-identity", BASE / "03-capture-core-closure/01-observe-pinned-build-inputs"):
    if str(directory) not in sys.path: sys.path.insert(0, str(directory))

from vulkan_docs_identity_members import DOCS_COMMIT
from vulkan_docs_identity_parse import F02
from vulkan_docs_observer_plan import SOURCE_TREE

Runner = Callable[[tuple[str, ...]], bytes]
NORMAL_MODES = frozenset(("100644", "100755"))
OBJECT_ID = frozenset("0123456789abcdef")


class SnapshotError(ValueError): pass


def _reject(message: str) -> None: raise SnapshotError(message)


def _system(command: tuple[str, ...]) -> bytes:
    try:
        result = subprocess.run(command, check=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except OSError as error:
        _reject(f"source Git command cannot run: {error}")
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        _reject(f"source Git command failed: {detail or result.returncode}")
    return result.stdout


def _run(command: tuple[str, ...], runner: Runner) -> bytes:
    try: value = runner(command)
    except SnapshotError: raise
    except Exception as error: _reject(f"source Git command cannot run: {error}")
    if not isinstance(value, bytes): _reject("source Git command returned non-bytes")
    return value


def _line(value: bytes, label: str) -> str:
    try: text = value.decode("ascii").rstrip("\n")
    except UnicodeDecodeError: _reject(f"source {label} is not ASCII")
    if not text or "\n" in text or "\r" in text: _reject(f"source {label} is not one line")
    return text


def _source_root(source: Path) -> Path:
    try:
        if source.is_symlink() or not source.is_dir(): _reject("source root is not a real directory")
        return source.resolve(strict=True)
    except OSError as error: _reject(f"source root cannot be resolved: {error}")


def _selector(value: bytes) -> str:
    try: result = value.decode("ascii")
    except UnicodeDecodeError: _reject("source tree selector is not ASCII")
    parsed = PurePosixPath(result)
    if not result or "\\" in result or parsed.is_absolute() or str(parsed) != result:
        _reject("source tree selector is invalid")
    if any(part in (".", "..") for part in parsed.parts): _reject("source tree selector escapes its root")
    return result


def _tree_rows(value: bytes) -> tuple[tuple[str, str, str], ...]:
    if not value or not value.endswith(b"\0"): _reject("source Git tree is not NUL-delimited")
    result = []
    for entry in value[:-1].split(b"\0"):
        try: header, name = entry.split(b"\t", 1); mode, kind, object_id = header.decode("ascii").split(" ")
        except (UnicodeDecodeError, ValueError): _reject("source Git tree has an invalid member")
        if mode not in NORMAL_MODES or kind != "blob": _reject("source Git tree has a non-regular member")
        if len(object_id) not in (40, 64) or set(object_id) - OBJECT_ID: _reject("source Git tree has an invalid object")
        result.append((_selector(name), object_id, mode))
    if not result or len({row[0] for row in result}) != len(result): _reject("source Git tree is empty or repeats a selector")
    return tuple(result)


def _summary(rows: list[dict[str, object]]) -> tuple[int, int, str]:
    rendered = json.dumps(rows, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")
    return len(rows), sum(row["bytes"] for row in rows), hashlib.sha256(rendered).hexdigest()


def source_snapshot(source: Path, runner: Runner = _system) -> dict[str, bytes]:
    """Return a complete, verified snapshot without reopening worktree members."""
    root = _source_root(source)
    if _run(("git", "-C", str(root), "status", "--porcelain", "--ignored"), runner):
        _reject("source worktree is not clean")
    commit = _line(_run(("git", "-C", str(root), "rev-parse", "HEAD"), runner), "commit")
    if commit != DOCS_COMMIT: _reject("source commit is not the reviewed Vulkan Docs commit")
    git_dir = _line(_run(("git", "-C", str(root), "rev-parse", "--absolute-git-dir"), runner), "Git directory")
    if not Path(git_dir).is_absolute(): _reject("source Git directory is not absolute")
    prefix = ("git", f"--git-dir={git_dir}")
    entries = _tree_rows(_run(prefix + ("ls-tree", "-r", "-z", DOCS_COMMIT), runner))
    payloads, rows = {}, []
    for selector, object_id, mode in entries:
        payload = _run(prefix + ("cat-file", "blob", object_id), runner)
        if len(payload) > F02.MAX_INPUT_BYTES: _reject("source Git blob exceeds the explicit input bound")
        payloads[selector] = payload
        rows.append({"selector": selector, "sha256": hashlib.sha256(payload).hexdigest(), "bytes": len(payload), "mode": mode})
    if _summary(rows) != SOURCE_TREE: _reject("source Git blobs do not reproduce the reviewed source tree")
    return payloads
