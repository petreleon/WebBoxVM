"""Finite, reproducible batch of all five GLES vertex/transform-feedback slices."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tempfile
from pathlib import Path

from .json_output import pretty_json
from .snapshot import SourceSnapshot
from .source import InventoryError, private, reject

ROOT = Path(__file__).resolve().parents[3]
SLICES = ROOT / ("todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/"
                 "02-command-object-state-raw-inventory/03-object-resource-command-slices/"
                 "06-vertex-transform-feedback-commands")
TASKS = (
    ("F03.3.2.2.3.6.1", "01-current-generic-attribute-templates", "gles_current_vertex_attribute_template"),
    ("F03.3.2.2.3.6.2", "02-vertex-array-attribute-binding-and-primitive-restart-commands", "gles_vertex_array_binding_command"),
    ("F03.3.2.2.3.6.3", "03-vertex-array-object-lifecycle-commands", "gles_vertex_array_object_lifecycle"),
    ("F03.3.2.2.3.6.4", "04-transform-feedback-object-declarations", "gles_transform_feedback_object"),
    ("F03.3.2.2.3.6.5", "05-transform-feedback-capture-control-declarations", "gles_transform_feedback_capture_control"),
)


def select_tasks(names: list[str]):
    if len(names) != len(set(names)) or set(names) - {item[0] for item in TASKS}:
        reject("batch selection contains duplicate or unknown tasks")
    return [item for item in TASKS if not names or item[0] in names]


def load_engine(task):
    identifier, folder, stem = task
    module = private(SLICES / folder / (stem + "_inventory.py"), identifier.replace(".", "_") + "_batch")
    return module.ENGINE


def collect(cache_root: Path, names: list[str], regenerate: bool = False, loader=load_engine):
    generated, items, source = [], [], None
    snapshot = SourceSnapshot(ROOT, cache_root)
    for task in select_tasks(names):
        engine = loader(task)
        engine.batch_snapshot = snapshot
        value = engine.rendered(cache_root) if regenerate else engine.validate(cache_root)
        engine.records(value)
        engine.artifact(engine.ARTIFACT.forbidden, value)
        if source is not None and not engine.exact(source, value["source"]):
            reject("batch slices do not use the same admitted source")
        source = value["source"]
        items.append({"task": task[0], "raw_entry_count": value["raw_entry_count"],
                      "inventory_sha256": value["inventory_sha256"],
                      "filename": engine.INVENTORY.name})
        generated.append((engine.INVENTORY.name, pretty_json(value) + "\n"))
    snapshot.finish()
    report = {"schema": 1, "kind": "webboxvm-gles-vertex-inventory-batch", "source": source,
              "mode": "regenerate" if regenerate else "check", "raw_only": True,
              "promotion_allowed": False, "task_count": len(items),
              "raw_entry_count": sum(item["raw_entry_count"] for item in items), "tasks": items}
    canonical = json.dumps(report, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()
    return {**report, "batch_sha256": hashlib.sha256(canonical).hexdigest()}, generated


def export(directory: Path, generated) -> None:
    if directory.is_symlink():
        reject("batch output directory must not be a symlink")
    names = [name for name, _ in generated]
    if len(names) != len(set(names)) or any(Path(name).name != name for name in names):
        reject("batch output names are duplicate or not local filenames")
    if any((directory / name).is_symlink() for name in names):
        reject("batch output must not replace a symlink")
    directory.mkdir(parents=True, exist_ok=True)
    staged = []
    try:
        for name, content in generated:
            with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=directory, delete=False) as handle:
                handle.write(content)
                staged.append((Path(handle.name), directory / name))
        for temporary, target in staged:
            os.replace(temporary, target)
    finally:
        for temporary, _ in staged:
            temporary.unlink(missing_ok=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache-root", type=Path, required=True)
    parser.add_argument("--task", action="append", default=[])
    parser.add_argument("--output-dir", type=Path, help="regenerate into a reviewable directory; leaves Git artifacts untouched")
    args = parser.parse_args()
    try:
        report, generated = collect(args.cache_root, args.task, args.output_dir is not None)
        if args.output_dir is not None:
            export(args.output_dir, generated)
        print(json.dumps(report, indent=2, sort_keys=True))
    except (InventoryError, OSError, ValueError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)


if __name__ == "__main__":
    main()
