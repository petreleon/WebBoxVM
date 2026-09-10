"""Descriptor-safe exact inventory for the staged input namespace only."""

from __future__ import annotations

import os
import stat

from vulkan_docs_input_cache import Cache
from vulkan_docs_input_model import InputMember, reject
from vulkan_docs_input_paths import receipt_relative, staged_relative, worktree_relative
from vulkan_docs_stage_model import StagingPlan, require_live_plan
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close


def _base(plan: StagingPlan) -> str:
    return f"{worktree_relative(plan)}/inputs"


def _expected(plan: StagingPlan, planned: tuple[InputMember, ...], receipt: bool) -> tuple[set[str], set[str]]:
    base = _base(plan) + "/"
    files = {staged_relative(plan, item).removeprefix(base) for item in planned}
    if receipt:
        files.add(receipt_relative(plan).removeprefix(base))
    directories = {""}
    for value in files:
        parts = value.split("/")
        directories.update("/".join(parts[:index]) for index in range(1, len(parts)))
    return files, directories


def _walk(
    descriptor: int, prefix: str, files: set[str], directories: set[str], maximum_entries: int, maximum_depth: int,
) -> None:
    _private(descriptor, "staged input directory")
    if len(files) + len(directories) >= maximum_entries:
        reject("Docs staged input inventory exceeds its entry bound")
    directories.add(prefix)
    try:
        entries = os.scandir(descriptor)
    except OSError as error:
        reject(f"Docs staged input inventory cannot enumerate a directory: {error}")
    with entries:
        for entry in entries:
            value = entry.name if not prefix else f"{prefix}/{entry.name}"
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError as error:
                reject(f"Docs staged input inventory cannot inspect {value}: {error}")
            if stat.S_ISREG(info.st_mode):
                if info.st_nlink != 1:
                    reject("Docs staged input inventory has a hard-link alias")
                if len(files) + len(directories) >= maximum_entries:
                    reject("Docs staged input inventory exceeds its entry bound")
                files.add(value)
                continue
            if not stat.S_ISDIR(info.st_mode) or value.count("/") + 1 > maximum_depth:
                reject("Docs staged input inventory has an unsafe member")
            try:
                child = os.open(entry.name, DIR_FLAGS, dir_fd=descriptor)
            except OSError as error:
                reject(f"Docs staged input inventory cannot open {value}: {error}")
            try:
                _walk(child, value, files, directories, maximum_entries, maximum_depth)
            finally:
                close(child)


def exact_inputs(plan: StagingPlan, cache: Cache, planned: tuple[InputMember, ...], *, receipt: bool) -> None:
    plan = require_live_plan(plan)
    base = _base(plan)
    expected_files, expected_directories = _expected(plan, planned, receipt)
    maximum_depth = max(value.count("/") + 1 for value in expected_directories if value)
    try:
        parent, leaf = cache._parent(base, False)
    except FileNotFoundError:
        reject("Docs staged input inventory is missing its input namespace")
    descriptor = -1
    try:
        descriptor = os.open(leaf, DIR_FLAGS, dir_fd=parent)
        files, directories = set(), set()
        _walk(descriptor, "", files, directories, len(expected_files) + len(expected_directories), maximum_depth)
        if files != expected_files or directories != expected_directories:
            reject("Docs staged input inventory is not exact")
        cache._same_parent(f"{base}/inventory", descriptor)
    except OSError as error:
        reject(f"Docs staged input inventory is unsafe: {error}")
    finally:
        close(descriptor)
        close(parent)
