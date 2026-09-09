"""Temporary byte-derived fixtures for successor-cache tests only."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
from dataclasses import dataclass
from pathlib import Path

HERE = Path(__file__).resolve().parent
IDENTITY_DIR = HERE.parent / "01-successor-identity"
FIXTURE = IDENTITY_DIR / "successor_identity.fixture.json"


def module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    value = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = value
    spec.loader.exec_module(value)
    return value


IDENTITY = module("f024_successor_cache_identity_fixture", IDENTITY_DIR / "successor_identity_contract.py")


@dataclass(frozen=True)
class FixtureBytes:
    path: Path
    raw: dict[str, bytes]
    generated: dict[str, bytes]


def canonical(value: object) -> str:
    raw = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(raw).hexdigest()


def cache_path(logical_id: str, kind: str, identifier: str, digest: str) -> str:
    suffix = "source" if kind == "raw" else "derived"
    return f"webboxvm-graphics/successor/{logical_id}/{kind}/{identifier}/{digest}.{suffix}"


def member(row: dict[str, object], payload: bytes, logical_id: str, kind: str) -> None:
    digest = hashlib.sha256(payload).hexdigest()
    row["sha256"], row["bytes"] = digest, len(payload)
    row["local_cache"] = cache_path(logical_id, kind, str(row["id"]), digest)


def generation(value: dict[str, object]) -> None:
    rows = value["members"]
    outputs = value["generations"][0]["output_member_ids"]
    selected = {row["id"]: row for row in rows if isinstance(row, dict)}
    tree = [
        {"id": identifier, "selector": selected[identifier]["selector"],
         "sha256": selected[identifier]["sha256"], "bytes": selected[identifier]["bytes"]}
        for identifier in outputs
    ]
    digest = canonical(tree)
    value["generations"][0]["output_tree_sha256"] = digest
    value["generations"][0]["clean_run_tree_sha256s"] = [digest, digest]


def seal(value: dict[str, object]) -> None:
    scope = value["scope"]
    scope["scope_sha256"] = canonical({key: item for key, item in scope.items() if key != "scope_sha256"})
    value["closure_sha256"] = canonical({key: item for key, item in value.items() if key != "closure_sha256"})


def build(root: Path) -> FixtureBytes:
    value = json.loads(FIXTURE.read_text(encoding="utf-8"))
    logical_id = value["required_input_id"]
    raw = {"fixture-root": b"fixture root raw\n", "fixture-registry": b"fixture registry raw\n"}
    generated = {
        "fixture-generated": b"fixture generated primary\n",
        "fixture-generated-extra": b"fixture generated secondary\n",
    }
    for row in value["members"][:2]:
        member(row, raw[row["id"]], logical_id, "raw")
    primary = value["members"][2]
    member(primary, generated[primary["id"]], logical_id, "generated")
    extra = dict(primary)
    extra.update(id="fixture-generated-extra", selector="generated/registry.adoc",
                 generated_code_role="fixture generated extra")
    member(extra, generated[extra["id"]], logical_id, "generated")
    value["members"].append(extra)
    outputs = ["fixture-generated", "fixture-generated-extra"]
    value["generations"][0]["output_member_ids"] = outputs
    value["scope"]["ordered_member_ids"] = [row["id"] for row in value["members"]]
    value["scope"]["generated_member_ids"] = outputs
    generation(value)
    seal(value)
    path = root / "successor-cache.fixture.json"
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    IDENTITY.validate(path)
    return FixtureBytes(path, raw, generated)


def build_interleaved(root: Path) -> FixtureBytes:
    """Build a valid closure whose one generation waits for a later raw producer."""
    result = build(root)
    value = json.loads(result.path.read_text(encoding="utf-8"))
    logical_id = value["required_input_id"]
    late_payload = b"fixture late raw\n"
    late = dict(value["members"][1])
    late.update(
        id="fixture-late-raw", selector="late.xml",
        immutable_url=str(late["immutable_url"]).replace("registry.xml", "late.xml"),
    )
    member(late, late_payload, logical_id, "raw")
    value["members"].insert(3, late)
    value["members"][4]["producer_member_ids"] = ["fixture-root", "fixture-late-raw"]
    value["scope"]["ordered_member_ids"] = [row["id"] for row in value["members"]]
    generation(value)
    seal(value)
    path = root / "successor-cache-interleaved.fixture.json"
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    IDENTITY.validate(path)
    return FixtureBytes(path, {**result.raw, "fixture-late-raw": late_payload}, result.generated)


def build_internal_producer(root: Path) -> FixtureBytes:
    """Build a valid generation with one output consuming another output internally."""
    result = build(root)
    value = json.loads(result.path.read_text(encoding="utf-8"))
    value["members"][3]["producer_member_ids"] = ["fixture-root", "fixture-generated"]
    seal(value)
    path = root / "successor-cache-internal-producer.fixture.json"
    path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
    IDENTITY.validate(path)
    return FixtureBytes(path, result.raw, result.generated)
