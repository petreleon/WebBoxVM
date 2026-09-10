"""Anchor proof-lineage raw reads to one sealed core-input capture."""

from __future__ import annotations

import sys
from dataclasses import dataclass
from pathlib import Path

from lineage_model import LineageError, reject
from lineage_parse import digest, identifier, member_bytes, raw_selector

HERE = Path(__file__).resolve().parent
if str(HERE.parent) not in sys.path: sys.path.insert(0, str(HERE.parent))
BIND = HERE.parent.parent / "03-capture-core-closure/02-bind-core-input-scope"
if str(BIND) not in sys.path: sys.path.insert(0, str(BIND))

from vulkan_docs_scope_bind import bind_capture
from vulkan_docs_scope_model import RAW as OBSERVED_RAW
from vulkan_docs_member_ids import raw_member_id as stable_raw_member_id

RAW_CAPTURE_FIELDS = frozenset(("kind", "selector", "revision", "immutable_url", "sha256", "bytes", "phase_roles"))


@dataclass(frozen=True)
class SealedScope:
    observation_sha256: str
    manifest_sha256: str
    scope_identity_sha256: str
    run_id: str
    raw_inputs: tuple[tuple[str, str, str, int], ...]

    def contains(self, input_id: str, selector: str, digest_value: str, byte_count: int) -> bool:
        return (input_id, selector, digest_value, byte_count) in self.raw_inputs


def raw_member_id(selector: object) -> str:
    try: return stable_raw_member_id(raw_selector(selector, "sealed raw selector"))
    except ValueError as error: reject(str(error))


def _identity(input_id: object, selector: object, digest_value: object, byte_count: object) -> tuple[str, str, str, int]:
    selected = raw_selector(selector, "sealed raw selector")
    if input_id != raw_member_id(selected): reject("sealed raw member id is not reversible")
    return str(input_id), selected, digest(digest_value, "sealed raw sha256"), member_bytes(byte_count, "sealed raw bytes")


def _rows(value: object) -> tuple[tuple[str, str, str, int], ...]:
    if not isinstance(value, list) or not value: reject("sealed scope has no raw records")
    rows = []
    for row in value:
        if not isinstance(row, dict) or set(row) != RAW_CAPTURE_FIELDS or row.get("kind") != OBSERVED_RAW:
            reject("sealed scope has an invalid raw record")
        rows.append(_identity(raw_member_id(row.get("selector")), row.get("selector"), row.get("sha256"), row.get("bytes")))
    if len(set(rows)) != len(rows) or len({item[0] for item in rows}) != len(rows):
        reject("sealed scope has colliding raw identities")
    return tuple(sorted(rows))


def checked(scope: object) -> SealedScope:
    if not isinstance(scope, SealedScope): reject("lineage lacks a sealed raw scope")
    digest(scope.observation_sha256, "sealed observation sha256")
    digest(scope.manifest_sha256, "sealed manifest sha256")
    digest(scope.scope_identity_sha256, "sealed scope identity sha256")
    identifier(scope.run_id, "sealed scope run id")
    if not isinstance(scope.raw_inputs, tuple) or not scope.raw_inputs or any(not isinstance(row, tuple) or len(row) != 4 for row in scope.raw_inputs):
        reject("sealed scope raw identities are invalid")
    rows = tuple(sorted(_identity(*row) for row in scope.raw_inputs))
    if tuple(sorted(scope.raw_inputs)) != rows: reject("sealed scope raw identities are not canonical")
    if len(set(rows)) != len(rows) or len({item[0] for item in rows}) != len(rows):
        reject("sealed scope raw identities collide")
    return scope


def capture_scope(observation: Path, artifact_root: Path, run_id: str) -> SealedScope:
    try: manifest = bind_capture(observation, artifact_root, run_id)
    except Exception as error: reject(f"sealed scope cannot bind its capture: {error}")
    return checked(SealedScope(manifest.capture.observation_digest, manifest.manifest_digest, manifest.scope_digest,
                               manifest.capture.identifier, _rows(manifest.value["raw_records"])))


def main() -> None:
    if len(sys.argv) != 4: raise SystemExit("usage: lineage_capture.py OBSERVATION ARTIFACT_ROOT RUN_ID")
    try: scope = capture_scope(Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3])
    except LineageError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"SCOPE: {len(scope.raw_inputs)} raw, proof-lineage-only-unadmitted")


if __name__ == "__main__": main()
