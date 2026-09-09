"""Validate pinned process provenance and observed input-event shapes."""

from __future__ import annotations

from vulkan_docs_observer_model import CONTENT_KINDS, PHASES, RUNTIME_PRODUCER, reject
from vulkan_docs_observer_parse import canonical, hex_bytes, path_selector, positive, source_limit

SOURCE = "/vulkan/"
GENERATED = "/work/generated/"
MAX_PID = 2**31 - 1
MAX_EVENT_BYTES = 64 * 1024 * 1024
MAX_MTIME_NS = 2**63 - 1


def executable(argv: tuple[str, ...]) -> str:
    return argv[0].rsplit("/", 1)[-1] if argv else ""


def phase(argv: tuple[str, ...]) -> str | None:
    program = executable(argv)
    if argv == RUNTIME_PRODUCER:
        return "producer"
    if program == "make":
        return "make-control"
    if program == "python3" and len(argv) > 1 and argv[1] == "/vulkan/scripts/genvk.py":
        return "generator"
    if program == "ruby" and len(argv) > 1 and executable((argv[1],)) == "asciidoctor":
        return "asciidoctor"
    if program == "node" and len(argv) > 1 and argv[1] == "/vulkan/scripts/translate_math.js":
        return "postprocess"
    if program == "cp":
        return "asset-copy"
    return None


def starts(rows: list[dict[str, object]]) -> dict[int, str]:
    result: dict[int, str] = {}
    producer_starts = 0
    for row in rows:
        if row.get("kind") != "start":
            continue
        if set(row) != {"kind", "pid", "argv_hex"}:
            reject("observer start has an invalid schema")
        identifier = positive(row.get("pid"), "observer start pid", MAX_PID)
        parts = tuple(part.decode("utf-8") for part in hex_bytes(row.get("argv_hex"), "observer argv").split(b"\0") if part)
        if not parts:
            reject("observer start is empty")
        current, previous = phase(parts) or "unknown", result.get(identifier)
        if previous not in (None, "unknown", current):
            reject("observer start changes an active process phase")
        if current != "unknown" or previous is None:
            result[identifier] = current
        producer_starts += parts == RUNTIME_PRODUCER
    if producer_starts != 1 or not PHASES <= set(result.values()):
        reject("observer did not start each exact pinned build phase")
    return result


def classify(path: str) -> tuple[str, str] | None:
    source = path_selector(path, SOURCE, "observed source selector")
    if source is not None:
        if source == ".git" or source.startswith(".git/"):
            return None
        if source.endswith(".pyc") or "/__pycache__/" in source:
            reject("observer resolved generated Python cache as a source input")
        return "raw-observed-input", source
    generated = path_selector(path, GENERATED, "observed generated selector")
    if generated is None:
        return None
    if generated == "out" or generated.startswith("out/"):
        return None
    if generated.endswith(".pyc") or "/__pycache__/" in generated:
        reject("observer resolved generated Python cache as an input")
    return "derived-observed-input", f"generated/{generated}"


def content(row: dict[str, object], processes: dict[int, str]) -> tuple[tuple[str, str] | None, str, tuple[int, int]]:
    kind = row.get("kind")
    if kind not in CONTENT_KINDS or set(row) != {"kind", "pid", "bytes", "path_hex", "observed_size", "observed_mtime_ns"}:
        reject("observer content event has an invalid schema")
    identifier = positive(row.get("pid"), "observer content pid", MAX_PID)
    role = processes.get(identifier)
    if role in (None, "unknown"):
        reject("observer content event has no declared process phase")
    positive(row.get("bytes"), "observer content bytes", MAX_EVENT_BYTES)
    try:
        path = hex_bytes(row.get("path_hex"), "observer content path").decode("ascii")
    except UnicodeDecodeError:
        reject("observer content path is not ASCII")
    member = classify(path)
    size = positive(row.get("observed_size"), "observer observed size", MAX_EVENT_BYTES)
    stamp = row.get("observed_mtime_ns")
    if type(stamp) is not int or abs(stamp) > MAX_MTIME_NS:
        reject("observer observed mtime is invalid")
    return member, role, (source_limit(size, "observer input size"), stamp) if member else (size, stamp)


def include(row: dict[str, object]) -> tuple[tuple[str, str], int]:
    if set(row) != {"file", "path", "line"} or type(row.get("line")) is not int or row["line"] < 1:
        reject("resolved include has an invalid schema")
    candidates = [item for item in (row.get("file"), row.get("path")) if isinstance(item, str)]
    mapped = [result for item in candidates if (result := classify(item)) is not None]
    if len(set(mapped)) != 1:
        reject("resolved include does not name exactly one input selector")
    return mapped[0], row["line"]


def producer_digest() -> str:
    return canonical(list(RUNTIME_PRODUCER), "webboxvm-graphics-vulkan-docs-runtime-producer-argv-v1")
