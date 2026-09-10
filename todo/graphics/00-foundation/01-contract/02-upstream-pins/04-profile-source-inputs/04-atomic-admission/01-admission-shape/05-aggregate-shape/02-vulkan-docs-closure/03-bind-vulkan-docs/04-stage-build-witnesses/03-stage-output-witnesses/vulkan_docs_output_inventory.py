"""Descriptor-safe inventory and rehashing for staged Docs output witnesses."""

from __future__ import annotations

import hashlib
import os
import stat

from vulkan_docs_output_model import MAX_OUTPUT_DEPTH, MAX_OUTPUT_ENTRIES, MAX_OUTPUT_MEMBER_BYTES, OutputMember, OutputTree, reject
from vulkan_docs_output_paths import output_base, receipt_relative, worktree_relative
from vulkan_docs_input_cache import Cache
from vulkan_docs_output_tree import tree, validate
from vulkan_docs_stage_fs import FILE_FLAGS, _same_read
from vulkan_docs_stage_model import RUN_IDS, StagingPlan, require_live_plan
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close, relative
def _base(plan: StagingPlan) -> str:
    return f"{worktree_relative(plan)}/outputs"
def _safe_directory(parent: int, name: str, label: str) -> tuple[int, object]:
    descriptor = -1
    try:
        before = os.stat(name, dir_fd=parent, follow_symlinks=False)
        descriptor = os.open(name, DIR_FLAGS, dir_fd=parent)
        _private(descriptor, label)
        _same_read(before, os.fstat(descriptor), label)
        return descriptor, before
    except (OSError, ValueError) as error:
        close(descriptor)
        reject(f"{label} is unsafe: {error}")
def _member(parent: int, name: str, info: object, selector: str) -> OutputMember:
    label = f"Docs staged output {selector}"
    try:
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
            reject(f"{label} is not an unaliased regular file")
        if info.st_size > MAX_OUTPUT_MEMBER_BYTES:
            reject(f"{label} exceeds its byte limit")
        descriptor = os.open(name, FILE_FLAGS, dir_fd=parent)
    except (OSError, ValueError) as error:
        reject(f"{label} is a symlink or unsafe: {error}")
    try:
        before = os.fstat(descriptor)
        _same_read(info, before, label)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
            reject(f"{label} is not an unaliased regular file")
        remaining, size, digest = before.st_size + 1, 0, hashlib.sha256()
        while remaining:
            payload = os.read(descriptor, min(1024 * 1024, remaining))
            if not payload:
                break
            size += len(payload)
            remaining -= len(payload)
            digest.update(payload)
        if not remaining:
            reject(f"{label} exceeds its byte limit")
        after = os.fstat(descriptor)
        _same_read(before, after, label)
        if after.st_nlink != 1 or size != before.st_size:
            reject(f"{label} changed during the operation")
        return OutputMember(selector, size, digest.hexdigest())
    except OSError as error:
        reject(f"{label} cannot be read safely: {error}")
    finally:
        close(descriptor)
def _walk_tree(
    cache: Cache, descriptor: int, base: str, prefix: str, members: list[OutputMember], directories: set[str],
) -> None:
    _private(descriptor, "staged output directory")
    if prefix.count("/") + 1 > MAX_OUTPUT_DEPTH: reject("Docs staged output inventory exceeds its depth bound")
    if len(members) + len(directories) >= MAX_OUTPUT_ENTRIES: reject("Docs staged output inventory exceeds its entry bound")
    directories.add(prefix)
    try:
        entries = os.scandir(descriptor)
    except OSError as error:
        reject(f"Docs staged output inventory cannot enumerate a directory: {error}")
    with entries:
        for entry in entries:
            selector = entry.name if not prefix else f"{prefix}/{entry.name}"
            relative(selector)
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError as error:
                reject(f"Docs staged output inventory cannot inspect {selector}: {error}")
            if stat.S_ISREG(info.st_mode):
                if len(members) + len(directories) >= MAX_OUTPUT_ENTRIES: reject("Docs staged output inventory exceeds its entry bound")
                members.append(_member(descriptor, entry.name, info, selector))
                continue
            if not stat.S_ISDIR(info.st_mode): reject("Docs staged output inventory has an unsafe member")
            child, before = _safe_directory(descriptor, entry.name, f"Docs staged output directory {selector}")
            try:
                _walk_tree(cache, child, base, selector, members, directories)
                _same_read(before, os.fstat(child), f"Docs staged output directory {selector}")
                cache._same_parent(f"{base}/{selector}/inventory", child)
            finally:
                close(child)
def _staged_tree(plan: StagingPlan, cache: Cache, run_id: str) -> OutputTree:
    base = output_base(plan, run_id)
    try:
        parent, leaf = cache._parent(base, False)
    except FileNotFoundError:
        reject(f"Docs staged output tree {run_id} is missing")
    descriptor = -1
    try:
        descriptor, before = _safe_directory(parent, leaf, f"Docs staged output tree {run_id}")
        members: list[OutputMember] = []
        _walk_tree(cache, descriptor, base, "", members, set())
        _same_read(before, os.fstat(descriptor), f"Docs staged output tree {run_id}")
        cache._same_parent(f"{base}/inventory", descriptor)
        return tree(run_id, tuple(sorted(members, key=lambda item: item.selector)))
    finally:
        close(descriptor)
        close(parent)
def staged_trees(plan: StagingPlan, cache: Cache) -> tuple[OutputTree, ...]:
    plan = require_live_plan(plan)
    values = tuple(_staged_tree(plan, cache, run_id) for run_id in RUN_IDS)
    return tuple(validate(plan, value) for value in values)
def _expected(plan: StagingPlan, trees: tuple[OutputTree, ...], receipt: bool) -> tuple[set[str], set[str]]:
    files = {f"{value.run_id}/{item.selector}" for value in trees for item in value.members}
    if receipt:
        files.add(receipt_relative(plan).removeprefix(_base(plan) + "/"))
    directories = {""}
    for value in files:
        parts = value.split("/")
        directories.update("/".join(parts[:index]) for index in range(1, len(parts)))
    return files, directories
def _inventory(descriptor: int, prefix: str, files: set[str], directories: set[str], maximum: int) -> None:
    _private(descriptor, "staged output directory")
    if prefix.count("/") > MAX_OUTPUT_DEPTH or len(files) + len(directories) >= maximum:
        reject("Docs staged output inventory exceeds its bound")
    directories.add(prefix)
    try:
        entries = os.scandir(descriptor)
    except OSError as error:
        reject(f"Docs staged output inventory cannot enumerate a directory: {error}")
    with entries:
        for entry in entries:
            value = entry.name if not prefix else f"{prefix}/{entry.name}"
            relative(value)
            try:
                info = entry.stat(follow_symlinks=False)
            except OSError as error:
                reject(f"Docs staged output inventory cannot inspect {value}: {error}")
            if stat.S_ISREG(info.st_mode):
                if info.st_nlink != 1 or len(files) + len(directories) >= maximum:
                    reject("Docs staged output inventory has an unsafe or excess file")
                files.add(value)
                continue
            if not stat.S_ISDIR(info.st_mode) or value.count("/") > MAX_OUTPUT_DEPTH:
                reject("Docs staged output inventory has an unsafe member")
            child, before = _safe_directory(descriptor, entry.name, f"Docs staged output directory {value}")
            try:
                _inventory(child, value, files, directories, maximum)
                _same_read(before, os.fstat(child), f"Docs staged output directory {value}")
            finally:
                close(child)
def exact_outputs(plan: StagingPlan, cache: Cache, trees: tuple[OutputTree, ...], receipt: bool) -> None:
    plan = require_live_plan(plan)
    if type(receipt) is not bool: reject("Docs staged output receipt flag is invalid")
    if not isinstance(trees, tuple) or tuple(value.run_id for value in trees if isinstance(value, OutputTree)) != RUN_IDS:
        reject("Docs staged output trees are invalid")
    if len(trees) != len(RUN_IDS) or not all(isinstance(value, OutputTree) for value in trees):
        reject("Docs staged output trees are invalid")
    expected_files, expected_directories = _expected(plan, tuple(validate(plan, value) for value in trees), receipt)
    base = _base(plan)
    try:
        parent, leaf = cache._parent(base, False)
    except FileNotFoundError:
        reject("Docs staged output inventory is missing its output namespace")
    descriptor = -1
    try:
        descriptor, before = _safe_directory(parent, leaf, "Docs staged output namespace")
        files, directories = set(), set(); maximum = MAX_OUTPUT_ENTRIES * len(RUN_IDS) + 2
        _inventory(descriptor, "", files, directories, maximum)
        _same_read(before, os.fstat(descriptor), "Docs staged output namespace")
        cache._same_parent(f"{base}/inventory", descriptor)
        if files != expected_files or directories != expected_directories:
            reject("Docs staged output inventory is not exact")
        if staged_trees(plan, cache) != trees:
            reject("Docs staged output trees do not match their recorded witnesses")
    finally:
        close(descriptor)
        close(parent)
