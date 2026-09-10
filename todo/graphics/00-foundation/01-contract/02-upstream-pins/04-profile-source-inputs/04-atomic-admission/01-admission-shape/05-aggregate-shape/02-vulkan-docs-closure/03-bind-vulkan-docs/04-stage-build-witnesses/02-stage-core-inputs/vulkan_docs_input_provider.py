"""Held-descriptor, paired provider reads for the recorded Docs input artifacts."""

from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
import hashlib
import os
import stat

from vulkan_docs_input_model import DERIVED, RAW, InputMember, reject
from vulkan_docs_input_paths import provider_root
from vulkan_docs_stage_model import StagingPlan, require_live_plan
from vulkan_docs_stage_paths import NOFOLLOW, _directory_fd, _private, child, close, relative

FILE_FLAGS = os.O_RDONLY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NONBLOCK", 0)


def _directory(base: int, value: str) -> int:
    try:
        descriptor = os.dup(base)
    except OSError as error:
        reject(f"Docs input provider root cannot be duplicated safely: {error}")
    try:
        for part in relative(value):
            next_descriptor = child(descriptor, part, False)
            try:
                _private(next_descriptor, "input provider directory")
            except BaseException:
                close(next_descriptor)
                raise
            close(descriptor)
            descriptor = next_descriptor
        return descriptor
    except BaseException:
        close(descriptor)
        raise


def _parent(base: int, value: str) -> tuple[int, str]:
    parts = relative(value)
    try:
        descriptor = os.dup(base)
    except OSError as error:
        reject(f"Docs input provider root cannot be duplicated safely: {error}")
    try:
        for part in parts[:-1]:
            next_descriptor = child(descriptor, part, False)
            try:
                _private(next_descriptor, "input provider directory")
            except BaseException:
                close(next_descriptor)
                raise
            close(descriptor)
            descriptor = next_descriptor
        return descriptor, parts[-1]
    except BaseException:
        close(descriptor)
        raise


def _capture_root(capture: object, kind: str) -> tuple[str, str]:
    if not isinstance(capture, dict):
        reject("Docs input provider capture is invalid")
    run_id, artifact = capture.get("run_id"), capture.get("artifact")
    if not isinstance(run_id, str) or not isinstance(artifact, str) or not artifact.startswith("runs/"):
        reject("Docs input provider capture is invalid")
    value = f"sources/{run_id}" if kind == RAW else f"{artifact}/generated"
    relative(value)
    return run_id, value


def _same(left, right, label: str) -> None:
    if (left.st_dev, left.st_ino, left.st_size, left.st_ctime_ns) != (right.st_dev, right.st_ino, right.st_size, right.st_ctime_ns):
        reject(f"{label} changed during the operation")


@dataclass
class Providers:
    roots: dict[tuple[str, str], int]

    def read(self, capture: object, member: InputMember) -> bytes:
        run_id, _ = _capture_root(capture, member.kind)
        descriptor = self.roots.get((run_id, member.kind))
        if descriptor is None:
            reject("Docs input provider is not one of the two selected captures")
        selector = member.selector if member.kind == RAW else member.selector.removeprefix("generated/")
        parent, leaf = _parent(descriptor, selector)
        label = f"Docs {run_id} {member.kind} provider {member.selector}"
        try:
            try:
                target = os.open(leaf, FILE_FLAGS, dir_fd=parent)
            except (OSError, ValueError) as error:
                reject(f"{label} is a symlink or unsafe: {error}")
            try:
                before = os.fstat(target)
                if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size != member.bytes:
                    reject(f"{label} is not its expected regular member")
                remaining, chunks = member.bytes + 1, []
                while remaining and (chunk := os.read(target, min(1024 * 1024, remaining))):
                    chunks.append(chunk)
                    remaining -= len(chunk)
                if not remaining:
                    reject(f"{label} exceeds its byte limit")
                payload = b"".join(chunks)
                after = os.fstat(target)
                _same(before, after, label)
                if after.st_nlink != 1:
                    reject(f"{label} gained a hard-link alias during its read")
                if len(payload) != member.bytes or hashlib.sha256(payload).hexdigest() != member.sha256:
                    reject(f"{label} does not match its bound bytes or sha256")
                return payload
            except OSError as error:
                reject(f"{label} cannot be read safely: {error}")
            finally:
                close(target)
        finally:
            close(parent)


@contextmanager
def providers(plan: StagingPlan, artifact_root: object):
    plan = require_live_plan(plan)
    root = provider_root(artifact_root)
    try:
        cache_alias = plan.external_root.exists() and root.samefile(plan.external_root)
    except OSError as error:
        reject(f"Docs input provider root cannot be compared safely: {error}")
    if root == plan.external_root or cache_alias:
        reject("Docs input provider root cannot be the cache root")
    try:
        descriptor = _directory_fd(root, False)
    except OSError as error:
        reject(f"Docs input provider root cannot be opened safely: {error}")
    opened: dict[tuple[str, str], int] = {}
    try:
        captures = plan.value.get("captures")
        if not isinstance(captures, list) or len(captures) != 2:
            reject("Docs input provider plan has invalid capture references")
        for capture in captures:
            for kind in (RAW, DERIVED):
                run_id, path = _capture_root(capture, kind)
                key = (run_id, kind)
                if key in opened:
                    reject("Docs input provider plan repeats a capture root")
                opened[key] = _directory(descriptor, path)
        try:
            identities = {(os.fstat(item).st_dev, os.fstat(item).st_ino) for item in opened.values()}
        except OSError as error:
            reject(f"Docs input provider roots cannot be inspected safely: {error}")
        if len(opened) != 4 or len(identities) != 4:
            reject("Docs input provider roots are missing or aliased")
        yield Providers(opened)
    finally:
        for item in opened.values():
            close(item)
        close(descriptor)
