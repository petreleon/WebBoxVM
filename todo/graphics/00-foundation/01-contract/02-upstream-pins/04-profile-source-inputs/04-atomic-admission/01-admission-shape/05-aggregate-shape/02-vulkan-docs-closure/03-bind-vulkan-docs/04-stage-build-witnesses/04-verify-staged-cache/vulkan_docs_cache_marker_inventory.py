"""One descriptor-safe exact inventory for a complete staged Docs closure."""

from __future__ import annotations

import os
import stat
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
OUTPUT = HERE.parent / "03-stage-output-witnesses"
STAGE = HERE.parent / "01-stage-contract"
for directory in (INPUT, OUTPUT, STAGE):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_bind import members as input_members
from vulkan_docs_input_cache import Cache
from vulkan_docs_input_paths import receipt_relative as input_receipt_relative
from vulkan_docs_input_paths import staged_relative as input_relative
from vulkan_docs_input_paths import worktree_relative
from vulkan_docs_output_paths import receipt_relative as output_receipt_relative
from vulkan_docs_output_paths import staged_relative as output_relative
from vulkan_docs_output_tree import manifests
from vulkan_docs_stage_fs import _same_read
from vulkan_docs_stage_marker import marker_relative
from vulkan_docs_stage_model import StagingPlan, reject, require_live_plan
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close, relative

INPUT_FILES, OUTPUT_FILES = 1761, 5061


def _inside(base: str, value: str) -> str:
    prefix = base + "/"
    if not value.startswith(prefix):
        reject("Docs staged closure member escapes its worktree")
    result = value.removeprefix(prefix)
    relative(result)
    return result


def _expected(plan: StagingPlan, planned: object, trees: object, marker: bool) -> tuple[set[str], set[str]]:
    plan = require_live_plan(plan)
    if type(marker) is not bool or not isinstance(planned, tuple) or planned != input_members(plan):
        reject("Docs staged closure has unbound inputs or marker state")
    if not isinstance(trees, tuple):
        reject("Docs staged closure has invalid output witnesses")
    manifests(plan, trees)
    base = worktree_relative(plan)
    files = {_inside(base, input_relative(plan, item)) for item in planned}
    files.add(_inside(base, input_receipt_relative(plan)))
    for tree in trees:
        files.update(_inside(base, output_relative(plan, tree.run_id, item.selector)) for item in tree.members)
    files.add(_inside(base, output_receipt_relative(plan)))
    if marker:
        files.add(_inside(base, marker_relative(plan)))
    expected = INPUT_FILES + OUTPUT_FILES + int(marker)
    if len(files) != expected:
        reject("Docs staged closure has an unexpected expected file count")
    directories = {""}
    for value in files:
        parts = value.split("/")
        directories.update("/".join(parts[:index]) for index in range(1, len(parts)))
    return files, directories


def _directory(parent: int, name: str, label: str) -> tuple[int, object]:
    descriptor = -1
    try:
        before = os.stat(name, dir_fd=parent, follow_symlinks=False)
        if not stat.S_ISDIR(before.st_mode):
            reject(f"{label} is not a directory")
        descriptor = os.open(name, DIR_FLAGS, dir_fd=parent)
        _private(descriptor, label)
        _same_read(before, os.fstat(descriptor), label)
        return descriptor, before
    except (OSError, ValueError) as error:
        close(descriptor)
        reject(f"{label} is unsafe: {error}")
    except BaseException:
        close(descriptor)
        raise


def _walk(
    cache: Cache, descriptor: int, base: str, prefix: str, files: set[str], directories: set[str],
    maximum: int, depth: int,
) -> None:
    _private(descriptor, "staged closure directory")
    if len(files) + len(directories) >= maximum:
        reject("Docs staged closure inventory exceeds its entry bound")
    directories.add(prefix)
    try:
        entries = os.scandir(descriptor)
    except OSError as error:
        reject(f"Docs staged closure inventory cannot enumerate {prefix or 'worktree'}: {error}")
    with entries:
        for entry in entries:
            value = entry.name if not prefix else f"{prefix}/{entry.name}"
            relative(value)
            if entry.name.startswith(".stage-"):
                reject("Docs staged closure inventory has a temporary member")
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError as error:
                reject(f"Docs staged closure inventory cannot inspect {value}: {error}")
            if stat.S_ISREG(info.st_mode):
                if info.st_nlink != 1:
                    reject("Docs staged closure inventory has a hard-link alias")
                if len(files) + len(directories) >= maximum:
                    reject("Docs staged closure inventory exceeds its entry bound")
                files.add(value)
                continue
            if not stat.S_ISDIR(info.st_mode) or value.count("/") + 1 > depth:
                reject("Docs staged closure inventory has an unsafe or over-deep member")
            child, before = _directory(descriptor, entry.name, f"Docs staged closure directory {value}")
            try:
                _walk(cache, child, base, value, files, directories, maximum, depth)
                _same_read(before, os.fstat(child), f"Docs staged closure directory {value}")
                cache._same_parent(f"{base}/{value}/inventory", child)
            finally:
                close(child)


def exact_closure(plan: StagingPlan, cache: Cache, planned: object, trees: object, marker: bool) -> None:
    """Reject any closure state besides the exact staged payload/receipt set."""
    plan = require_live_plan(plan)
    files, directories = _expected(plan, planned, trees, marker)
    base = worktree_relative(plan)
    depth = max(value.count("/") + 1 for value in directories if value)
    try:
        parent, leaf = cache._parent(base, False)
    except FileNotFoundError:
        reject("Docs staged closure inventory is missing its worktree")
    descriptor = -1
    try:
        descriptor, before = _directory(parent, leaf, "Docs staged closure worktree")
        found_files, found_directories = set(), set()
        _walk(cache, descriptor, base, "", found_files, found_directories, len(files) + len(directories), depth)
        _same_read(before, os.fstat(descriptor), "Docs staged closure worktree")
        cache._same_parent(f"{base}/inventory", descriptor)
        if found_files != files or found_directories != directories:
            reject("Docs staged closure inventory is not exact")
    finally:
        close(descriptor)
        close(parent)
