"""Fixed schemas for the unadmitted Vulkan Docs input observer."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
import sys

IDENTITY = Path(__file__).resolve().parent.parent.parent / "02-actual-closure-identity"
if str(IDENTITY) not in sys.path:
    sys.path.insert(0, str(IDENTITY))

from vulkan_docs_identity_build import ARGV  # noqa: E402

RAW = "raw-observed-input"
DERIVED = "derived-observed-input"
PHASES = frozenset(("producer", "make-control", "generator", "asciidoctor", "postprocess", "asset-copy"))
CONTENT_KINDS = frozenset(("read", "pread", "readv", "mmap", "fread", "fgets", "copy", "sendfile", "splice"))
MAX_EVENTS = 200_000
MAX_RECORDS = 8_192
RUNTIME_PRODUCER = ("python3", *ARGV)

RECORD_FIELDS = frozenset(("kind", "selector", "sha256", "bytes", "phase_roles"))
RUN_FIELDS = frozenset((
    "id", "artifact", "source_tree_sha256", "generated_tree_sha256", "primary_html_sha256",
    "io_trace_sha256", "include_trace_sha256", "input_manifest_sha256", "include_identity_sha256", "producer_argv_sha256",
    "raw_count", "derived_count", "include_count", "phase_counts", "run_sha256",
))
OBSERVATION_FIELDS = frozenset((
    "schema", "contract", "status", "profile", "role", "required_input_id", "build_witness_sha256",
    "observer_source_sha256", "runs", "observation_sha256",
))


class ObserverError(ValueError):
    """The replay observer is malformed, incomplete, or outside its declared scope."""


def reject(message: str) -> None:
    raise ObserverError(message)


@dataclass(frozen=True)
class InputRecord:
    kind: str
    selector: str
    digest: str
    byte_count: int
    phases: tuple[str, ...]


@dataclass(frozen=True)
class ObservationRun:
    identifier: str
    digest: str
    raw_count: int
    derived_count: int
    include_count: int


@dataclass(frozen=True)
class Observation:
    digest: str
    runs: tuple[ObservationRun, ...]
    state: str = field(default="input-observation-only-unadmitted", init=False)
    admitted: bool = field(default=False, init=False)
    cutover_ready: bool = field(default=False, init=False)
