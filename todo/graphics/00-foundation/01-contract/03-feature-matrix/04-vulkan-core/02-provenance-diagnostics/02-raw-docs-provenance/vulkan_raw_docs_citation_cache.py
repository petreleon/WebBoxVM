"""Immutable external cache contract for reviewed raw Vulkan-Docs fragments."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import stat
import sys
import tempfile
from pathlib import Path, PurePosixPath
from urllib.request import urlopen

HERE = Path(__file__).resolve().parent
def fixed_source():
    path, name = HERE / "vulkan_raw_docs_citation_sources.py", "f03422_fixed_citation_sources"
    prior = sys.modules.get(name)
    try:
        if path.is_symlink() or not path.is_file():
            raise RuntimeError("fixed citation sources must be a regular file")
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load fixed citation sources")
        module = importlib.util.module_from_spec(spec); sys.modules[name] = module; spec.loader.exec_module(module)
        if Path(getattr(module, "__file__", "")).resolve() != path.resolve():
            raise RuntimeError("citation sources resolved from an unexpected path")
        return module
    finally:
        if prior is None:
            sys.modules.pop(name, None)
        else:
            sys.modules[name] = prior
SOURCE = fixed_source()
CitationSourceError, reject, canonical = SOURCE.CitationSourceError, SOURCE.reject, SOURCE.canonical
REVISION, MAX_BYTES = "f84d432d5b8912362f96f581f29bbc4f3c8c7843", 8 * 1024 * 1024
ROOT = {"id": "vulkan-14-spec", "kind": "upstream-source", "scope": "normative-source", "revision": REVISION,
        "sha256": "069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0", "bytes": 8685,
        "license": "CC-BY-4.0 (vkspec.adoc SPDX-License-Identifier)",
        "attribution": "Copyright 2014-2026 The Khronos Group Inc.; Vulkan-Docs vkspec.adoc; CC-BY-4.0",
        "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/vkspec.adoc",
        "artifact": "webboxvm-graphics/f02/vulkan-14-spec/069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0.source"}
FRAGMENTS = (
    {"id": "raw-docs-introduction", "root_id": "vulkan-14-spec", "revision": REVISION,
     "raw_path": "chapters/introduction.adoc", "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/chapters/introduction.adoc", "sha256": "a8cff7be42b43f9cab6c150af845587d4d4f3758a4266972290735ed55ac9767", "bytes": 15959,
     "license": "CC-BY-4.0 (raw fragment SPDX-License-Identifier)",
     "attribution": "Copyright 2015-2026 The Khronos Group Inc.; Vulkan-Docs chapters/introduction.adoc; CC-BY-4.0",
     "semantic_anchor": "[[introduction]]"},
    {"id": "raw-docs-fundamentals-execmodel", "root_id": "vulkan-14-spec", "revision": REVISION,
     "raw_path": "chapters/fundamentals.adoc", "immutable_url": f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/chapters/fundamentals.adoc", "sha256": "70dde6bad83bd48c3438eb96a9cfc3387f53dfbdd8a1cae33a87dab1310cc0d3", "bytes": 91224,
     "license": "CC-BY-4.0 (raw fragment SPDX-License-Identifier)",
     "attribution": "Copyright 2015-2026 The Khronos Group Inc.; Vulkan-Docs chapters/fundamentals.adoc; CC-BY-4.0",
     "semantic_anchor": "[[fundamentals-execmodel]]"},
)


def raw_path(value: object) -> PurePosixPath:
    if not isinstance(value, str) or not value:
        reject("raw Docs path is invalid")
    path = PurePosixPath(value)
    if (path.is_absolute() or path.as_posix() != value or ".." in path.parts
            or any(part in {".", "generated", "html", "man", "out"} for part in path.parts) or value.endswith(".html")):
        reject("raw Docs path escapes the reviewed non-generated source set")
    return path


def manifest() -> dict[str, object]:
    return {"schema": 1, "kind": "webboxvm-vulkan-raw-docs-citation-cache", "root": {"raw_path": "vkspec.adoc", **ROOT},
            "fragments": list(FRAGMENTS), "include_closure": "unadmitted-not-claimed"}


def external_root(path: Path, fresh: bool = False) -> Path:
    if not isinstance(path, Path) or not path.is_absolute() or ".." in path.parts:
        reject("raw Docs cache root must be an absolute nonsymlink path")
    current = Path(path.anchor)
    for part in path.parts[1:]:
        current /= part
        if current.is_symlink():
            reject("raw Docs cache root has a symlinked ancestor")
    root = path.resolve()
    if root == SOURCE.REPO or SOURCE.REPO in root.parents:
        reject("raw Docs cache must remain external to the repository")
    if fresh:
        if root.exists() and (not root.is_dir() or any(root.iterdir())):
            reject("fresh raw Docs cache root must be missing or empty")
        if root.parent.is_symlink() or not root.parent.is_dir():
            reject("raw Docs cache parent must be a regular directory")
    elif not root.is_dir() or root.is_symlink():
        reject("raw Docs cache root must be an existing regular directory")
    return root


def target(root: Path, relative: object) -> Path:
    path = root
    for part in raw_path(relative).parts:
        path /= part
        if path.is_symlink():
            reject("raw Docs cache contains a symlinked path")
    return path


def secure_bytes(path: Path, label: str) -> bytes:
    try:
        before = os.lstat(path)
        if not stat.S_ISREG(before.st_mode) or not hasattr(os, "O_NOFOLLOW") or not hasattr(os, "O_NONBLOCK"):
            reject(f"{label} must be a regular no-follow file")
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        try:
            after = os.fstat(descriptor)
            if not stat.S_ISREG(after.st_mode) or (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino):
                reject(f"{label} changed while opening")
            stream = os.fdopen(descriptor, "rb", closefd=False)
            try:
                return stream.read(MAX_BYTES + 1)
            finally:
                stream.close()
        finally:
            os.close(descriptor)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")


def cache_document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(secure_bytes(path, "raw Docs cache manifest").decode("utf-8"), object_pairs_hook=SOURCE.pairs)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        reject(f"raw Docs cache manifest cannot be parsed: {error}")
    if not isinstance(value, dict):
        reject("raw Docs cache manifest is not a JSON object")
    return value


def payload(path: Path, expected: dict[str, object], anchor: object = None) -> None:
    data = secure_bytes(path, "raw Docs payload")
    if (len(data) > MAX_BYTES or (len(data), hashlib.sha256(data).hexdigest()) != (expected["bytes"], expected["sha256"])
            or b"// SPDX-License-Identifier: CC-BY-4.0" not in data.splitlines()[:5]):
        reject("raw Docs payload has an unexpected identity or license")
    if anchor is not None and (not isinstance(anchor, str) or data.count(anchor.encode("utf-8")) != 1):
        reject("raw Docs semantic anchor is missing or ambiguous")


def cache(cache_root: Path) -> dict[str, object]:
    root, expected = external_root(cache_root), manifest()
    if cache_document(root / "citation-cache.json") != expected:
        reject("raw Docs cache has a mixed revision, license, or fragment manifest")
    found = set()
    for path in root.rglob("*"):
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            reject("raw Docs cache contains a nonregular path")
        if path.is_file():
            found.add(path.relative_to(root).as_posix())
    expected_paths = {"citation-cache.json", "vkspec.adoc", *(item["raw_path"] for item in FRAGMENTS)}
    if found != expected_paths:
        reject("raw Docs cache has an unexpected, generated, or missing payload")
    payload(target(root, "vkspec.adoc"), ROOT)
    for item in FRAGMENTS:
        payload(target(root, item["raw_path"]), item, item["semantic_anchor"])
    return {"mode": "sealed-external-reviewed-raw-fragments", "file_count": len(found),
            "manifest_sha256": hashlib.sha256(canonical(expected)).hexdigest()}


def refresh(cache_root: Path, timeout: float = 30.0) -> dict[str, object]:
    final = external_root(cache_root, fresh=True)
    if timeout <= 0:
        reject("raw Docs cache timeout must be positive")
    with tempfile.TemporaryDirectory(prefix=f".{final.name}.", dir=final.parent) as temporary:
        stage = Path(temporary) / "cache"; stage.mkdir()
        for path, item in (("vkspec.adoc", ROOT), *((value["raw_path"], value) for value in FRAGMENTS)):
            try:
                with urlopen(f"https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/{REVISION}/{path}", timeout=timeout) as response:
                    data = response.read(MAX_BYTES + 1)
            except OSError as error:
                reject(f"raw Docs refresh failed: {error}")
            destination = target(stage, path); destination.parent.mkdir(parents=True, exist_ok=True); destination.write_bytes(data)
            payload(destination, item, item.get("semantic_anchor"))
        (stage / "citation-cache.json").write_bytes(canonical(manifest()))
        cache(stage); os.replace(stage, final)
    return cache(final)
