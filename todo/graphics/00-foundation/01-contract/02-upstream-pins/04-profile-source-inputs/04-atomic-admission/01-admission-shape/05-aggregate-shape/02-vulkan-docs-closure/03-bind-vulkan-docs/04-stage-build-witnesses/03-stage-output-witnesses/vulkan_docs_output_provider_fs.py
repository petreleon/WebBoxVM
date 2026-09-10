"""Descriptor-safe recursive scan and re-read helpers for Docs outputs."""

from __future__ import annotations

import hashlib
import os
import stat
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "02-stage-core-inputs"
OBSERVER = HERE.parent.parent / "03-capture-core-closure/01-observe-pinned-build-inputs"
for directory in (INPUT, OBSERVER):
    if str(directory) not in sys.path:
        sys.path.insert(0, str(directory))

from vulkan_docs_input_provider import FILE_FLAGS, _directory, _parent, _same
from vulkan_docs_observer_contract import run as observer_run
from vulkan_docs_output_model import MAX_OUTPUT_DEPTH, MAX_OUTPUT_ENTRIES, MAX_OUTPUT_MEMBER_BYTES, OutputMember, OutputTree, decode, member
from vulkan_docs_output_tree import tree
from vulkan_docs_stage_model import MAX_OUTPUT_BYTES, MAX_OUTPUT_FILES, MAX_PLAN_BYTES, reject
from vulkan_docs_stage_paths import DIR_FLAGS, _private, close, relative


def _info(value: os.stat_result, label: str) -> None:
    if value.st_uid != os.geteuid() or value.st_mode & 0o022:
        reject(f"{label} is not private to the current user")


def _read(parent: int, leaf: str, selector: str, limit: int, expected: OutputMember | None = None) -> tuple[bytes, OutputMember]:
    relative(selector)
    try:
        descriptor = os.open(leaf, FILE_FLAGS, dir_fd=parent)
    except (OSError, ValueError) as error:
        reject(f"Docs output provider {selector} is a symlink or unsafe: {error}")
    try:
        before = os.fstat(descriptor)
        _info(before, f"Docs output provider {selector}")
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > limit:
            reject(f"Docs output provider {selector} is not a bounded unaliased regular file")
        remaining, chunks = before.st_size + 1, []
        while remaining and (chunk := os.read(descriptor, min(1024 * 1024, remaining))):
            chunks.append(chunk)
            remaining -= len(chunk)
        payload, after = b"".join(chunks), os.fstat(descriptor)
        _same(before, after, f"Docs output provider {selector}")
        if after.st_nlink != 1 or not remaining or len(payload) != before.st_size:
            reject(f"Docs output provider {selector} changed its byte count during read")
        result = member(selector, len(payload), hashlib.sha256(payload).hexdigest())
        if expected is not None and result != expected:
            reject(f"Docs output provider {selector} does not match its scanned witness")
        return payload, result
    except OSError as error:
        reject(f"Docs output provider {selector} cannot be read safely: {error}")
    finally:
        close(descriptor)


def _read_parent(root: int, value: OutputMember) -> bytes:
    parent, leaf = _parent(root, value.selector)
    try:
        return _read(parent, leaf, value.selector, MAX_OUTPUT_MEMBER_BYTES, value)[0]
    finally:
        close(parent)


def _run(root: int, capture: dict[str, object]) -> bytes:
    artifact = capture.get("artifact")
    if not isinstance(artifact, str):
        reject("Docs output provider capture lacks its artifact")
    selector = f"{artifact}/observer/run.json"
    parent, leaf = _parent(root, selector)
    try:
        payload, _ = _read(parent, leaf, selector, MAX_PLAN_BYTES)
    finally:
        close(parent)
    try:
        value = decode(payload)
        parsed = observer_run(value)
    except Exception as error:
        reject(f"Docs output provider run witness is invalid: {error}")
    pairs = (("id", "run_id"), ("artifact", "artifact"), ("run_sha256", "run_sha256"),
             ("source_tree_sha256", "source_tree_sha256"), ("generated_tree_sha256", "generated_tree_sha256"),
             ("primary_html_sha256", "primary_html_sha256"), ("io_trace_sha256", "io_trace_sha256"),
             ("include_trace_sha256", "include_trace_sha256"), ("producer_argv_sha256", "producer_argv_sha256"))
    if any(value.get(left) != capture.get(right) for left, right in pairs) or (parsed.identifier, parsed.digest) != (capture["run_id"], capture["run_sha256"]):
        reject("Docs output provider run witness does not bind the selected capture")
    return payload


def _scan(descriptor: int, run_id: str) -> OutputTree:
    rows: list[OutputMember] = []
    state = [0, 0]

    def walk(current: int, prefix: str) -> None:
        _private(current, "output provider directory")
        try:
            entries = os.scandir(current)
        except OSError as error:
            reject(f"Docs output provider cannot enumerate {prefix or 'generated'}: {error}")
        with entries:
            for entry in sorted(entries, key=lambda item: item.name):
                selector = entry.name if not prefix else f"{prefix}/{entry.name}"
                relative(selector)
                state[0] += 1
                if state[0] > MAX_OUTPUT_ENTRIES or selector.count("/") + 1 > MAX_OUTPUT_DEPTH:
                    reject("Docs output provider tree exceeds its explicit entry or depth bound")
                try:
                    info = entry.stat(follow_symlinks=False)
                except OSError as error:
                    reject(f"Docs output provider cannot inspect {selector}: {error}")
                _info(info, f"Docs output provider {selector}")
                if stat.S_ISREG(info.st_mode):
                    if info.st_nlink != 1 or len(rows) >= MAX_OUTPUT_FILES or state[1] + info.st_size > MAX_OUTPUT_BYTES:
                        reject("Docs output provider tree exceeds its regular-file bound")
                    state[1] += info.st_size
                    rows.append(_read(current, entry.name, selector, MAX_OUTPUT_MEMBER_BYTES)[1])
                elif stat.S_ISDIR(info.st_mode):
                    try:
                        nested = os.open(entry.name, DIR_FLAGS, dir_fd=current)
                    except OSError as error:
                        reject(f"Docs output provider directory {selector} is unsafe: {error}")
                    try:
                        walk(nested, selector)
                    finally:
                        close(nested)
                else:
                    reject("Docs output provider tree has a symlink or non-regular member")

    walk(descriptor, "")
    return tree(run_id, tuple(sorted(rows, key=lambda item: item.selector)))
