"""Public held-provider boundary for the two Docs output witnesses."""

from __future__ import annotations

from contextlib import contextmanager
import os
from pathlib import Path

from vulkan_docs_output_model import OutputMember, OutputTree
from vulkan_docs_output_paths import provider_relative, provider_root, source_relative
from vulkan_docs_output_provider_fs import _directory, _read_parent, _run, _scan
from vulkan_docs_stage_model import RUN_IDS, StagingPlan, reject, require_live_plan
from vulkan_docs_stage_paths import _directory_fd, _private, close


def _identity(left: int, right: int) -> tuple[int, int, int, int]:
    try:
        return os.fstat(left).st_dev, os.fstat(left).st_ino, os.fstat(right).st_dev, os.fstat(right).st_ino
    except OSError as error:
        reject(f"Docs output provider root cannot be inspected safely: {error}")


class Providers:
    """One held descriptor set; tree equality never collapses recorded origins."""

    def __init__(self, plan: StagingPlan, root: Path, descriptor: int, roots: dict[tuple[str, str], int], runs: dict[str, bytes]) -> None:
        self.plan, self.root, self.descriptor, self.roots, self.runs, self.trees = plan, root, descriptor, roots, runs, {}

    def _capture(self, capture: dict[str, object]) -> str:
        if type(capture) is not dict or capture.get("run_id") not in RUN_IDS or capture not in self.plan.value["captures"]:
            reject("Docs output provider is not one of the selected captures")
        return capture["run_id"]

    def _roots(self) -> None:
        current = _directory_fd(self.root, False)
        try:
            _private(current, "output provider root")
            identity = _identity(current, self.descriptor)
            if identity[:2] != identity[2:]:
                reject("Docs output provider root changed during staging")
            for run_id in RUN_IDS:
                for kind, value in (("generated", provider_relative(self.plan, run_id)), ("source", source_relative(self.plan, run_id))):
                    named = _directory(current, value)
                    try:
                        identity = _identity(named, self.roots[(run_id, kind)])
                        if identity[:2] != identity[2:]:
                            reject("Docs output provider capture root changed during staging")
                    finally:
                        close(named)
        finally:
            close(current)

    def scan(self, capture: dict[str, object]) -> OutputTree:
        run_id = self._capture(capture)
        result = _scan(self.roots[(run_id, "generated")], run_id)
        self.trees[run_id] = result
        return result

    def read(self, capture: dict[str, object], value: OutputMember) -> bytes:
        run_id = self._capture(capture)
        known = self.trees.get(run_id)
        if not isinstance(value, OutputMember) or known is None or value not in known.members:
            reject("Docs output provider member was not scanned for this capture")
        return _read_parent(self.roots[(run_id, "generated")], value)

    def confirm(self) -> None:
        self._roots()
        for capture in self.plan.value["captures"]:
            run_id = self._capture(capture)
            if _run(self.descriptor, capture) != self.runs[run_id]:
                reject("Docs output provider run witness changed during staging")
            if run_id in self.trees and _scan(self.roots[(run_id, "generated")], run_id) != self.trees[run_id]:
                reject("Docs output provider tree changed during staging")
        self._roots()


@contextmanager
def providers(plan: StagingPlan, artifact_root: object):
    plan, root = require_live_plan(plan), provider_root(artifact_root)
    try:
        alias = plan.external_root.exists() and root.samefile(plan.external_root)
    except OSError as error:
        reject(f"Docs output provider root cannot be compared safely: {error}")
    if root == plan.external_root or alias:
        reject("Docs output provider root cannot be the cache root")
    descriptor, roots = _directory_fd(root, False), {}
    try:
        captures = plan.value.get("captures")
        if not isinstance(captures, list) or len(captures) != len(RUN_IDS) or not all(type(item) is dict for item in captures) or tuple(item.get("run_id") for item in captures) != RUN_IDS:
            reject("Docs output provider plan has invalid capture references")
        for capture in captures:
            run_id = capture["run_id"]
            roots[(run_id, "generated")] = _directory(descriptor, provider_relative(plan, run_id))
            roots[(run_id, "source")] = _directory(descriptor, source_relative(plan, run_id))
        identities = {_identity(value, value)[:2] for value in roots.values()}
        if len(roots) != 4 or len(identities) != 4:
            reject("Docs output provider roots are missing or aliased")
        runs = {capture["run_id"]: _run(descriptor, capture) for capture in captures}
        yield Providers(plan, root, descriptor, roots, runs)
    finally:
        for value in roots.values():
            close(value)
        close(descriptor)
