"""Small hermetic observer inputs; never an actual Docs closure."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from vulkan_docs_observer_model import OBSERVATION_FIELDS, PHASES, RUN_FIELDS, RUNTIME_PRODUCER
from vulkan_docs_observer_parse import canonical
from vulkan_docs_observer_plan import OUTPUT, SOURCE_TREE, TREE, observer_digest, producer_digest, witness_digest


def write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def encode(path: str) -> str:
    return path.encode("ascii").hex()


def dump(path: Path, rows: list[dict[str, object]]) -> None:
    path.write_text("\n".join(json.dumps(row, separators=(",", ":")) for row in rows) + "\n", encoding="utf-8")


def source_tree(root: Path) -> tuple[Path, Path]:
    source, generated = root / "source", root / "generated"
    for name, data in {
        "vkspec.adoc": b"root", "scripts/genvk.py": b"generator", "config/khronos.css": b"style",
        "katex/katex.min.js": b"katex",
    }.items():
        write(source / name, data)
    write(generated / "specattribs.adoc", b"attributes")
    write(generated / "out/html/vkspec.html", b"output")
    return source, generated


def traces(root: Path) -> tuple[Path, Path]:
    io, includes = root / "io.jsonl", root / "includes.jsonl"
    starts = ((11, RUNTIME_PRODUCER), (12, ("make",)), (13, ("python3", "/vulkan/scripts/genvk.py")),
              (14, ("ruby", "asciidoctor")), (15, ("node", "/vulkan/scripts/translate_math.js")), (16, ("cp", "-rf")))
    rows = [{"kind": "start", "pid": pid, "argv_hex": b"\0".join(item.encode() for item in argv).hex()} for pid, argv in starts]
    paths = ((11, "/vulkan/vkspec.adoc"), (12, "/vulkan/config/khronos.css"), (13, "/vulkan/scripts/genvk.py"),
             (14, "/vulkan/vkspec.adoc"), (14, "/work/generated/specattribs.adoc"), (15, "/vulkan/katex/katex.min.js"),
             (16, "/vulkan/katex/katex.min.js"), (15, "/work/generated/out/html/vkspec.html"))
    def event(pid: int, name: str) -> dict[str, object]:
        relative = name.removeprefix("/vulkan/") if name.startswith("/vulkan/") else name.removeprefix("/work/generated/")
        location = root / ("source" if name.startswith("/vulkan/") else "generated") / relative
        state = location.stat()
        return {"kind": "fgets" if pid == 12 else "read", "pid": pid, "bytes": 1, "path_hex": encode(name),
                "observed_size": state.st_size, "observed_mtime_ns": state.st_mtime_ns}
    rows.extend(event(pid, name) for pid, name in paths)
    dump(io, rows)
    dump(includes, [{"file": "/vulkan/vkspec.adoc", "path": "/vulkan/vkspec.adoc", "line": 1},
                    {"file": "/work/generated/specattribs.adoc", "path": "/work/generated/specattribs.adoc", "line": 2}])
    return io, includes


def sealed_run(identifier: str, salt: str) -> dict[str, object]:
    value = {
        "id": identifier, "artifact": f"runs/{identifier}", "source_tree_sha256": SOURCE_TREE[2],
        "generated_tree_sha256": TREE[2], "primary_html_sha256": OUTPUT["sha256"],
        "io_trace_sha256": "1" * 64, "include_trace_sha256": "2" * 64, "input_manifest_sha256": "3" * 64,
        "include_identity_sha256": "4" * 64, "producer_argv_sha256": producer_digest(), "raw_count": 4,
        "derived_count": 1, "include_count": 2,
        "phase_counts": {name: 1 for name in PHASES},
    }
    assert set(value) == RUN_FIELDS - {"run_sha256"}
    value["run_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-observation-run-v1")
    return value


def observation() -> dict[str, object]:
    value = {
        "schema": 1, "contract": "vulkan-docs-core-input-observation-v1", "status": "input-observation-only-unadmitted",
        "profile": "vulkan-1.4-core", "role": "api-limit-format-spec", "required_input_id": "vulkan-14-spec",
        "build_witness_sha256": witness_digest(), "observer_source_sha256": observer_digest(),
        "runs": [sealed_run("observer-a", "a"), sealed_run("observer-b", "b")],
    }
    assert set(value) == OBSERVATION_FIELDS - {"observation_sha256"}
    value["observation_sha256"] = canonical(value, "webboxvm-graphics-vulkan-docs-observation-v1")
    return value
