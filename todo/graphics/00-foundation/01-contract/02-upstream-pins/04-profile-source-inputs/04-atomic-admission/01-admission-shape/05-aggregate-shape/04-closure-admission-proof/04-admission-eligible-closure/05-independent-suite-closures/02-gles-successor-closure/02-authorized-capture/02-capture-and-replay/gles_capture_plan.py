"""Derive the exact raw GLES capture plan and validate its mustpass root."""

from __future__ import annotations

import importlib.util
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
FETCH_DIR = HERE.parents[9] / "02-fetch-verifier" / "01-fetch-contract"
CONTRACT_DIR = HERE.parent / "01-closure-contract"
sys.path.insert(0, str(FETCH_DIR))
from source_model import ContractError, SourceInput, reject


def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


CONTRACT = reviewed("f025_gles_capture_contract", CONTRACT_DIR / "gles_successor_contract.py")


@dataclass(frozen=True)
class CapturePlan:
    contract_sha256: str
    closure_sha256: str
    configuration_sha256: str
    sources: tuple[SourceInput, ...]
    roles: tuple[str, ...]
    core_configurations: tuple[dict[str, object], ...]
    excluded_configurations: tuple[dict[str, object], ...]

    @property
    def root(self) -> SourceInput:
        return self.sources[0]


def source_input(record: dict[str, object], role: str, contract: dict[str, object]) -> SourceInput:
    source, closure = contract["source"], contract["closure"]
    identifier = record["id"]
    if not isinstance(identifier, str) or not isinstance(source, dict) or not isinstance(closure, dict):
        reject("GLES contract source record is malformed")
    family = source["root_source_family"] if role == "root" else source["member_source_family"]
    entry = {"id": identifier, "source_family": family,
             "immutable_url": str(source["raw_url_prefix"]) + str(record["selector"]),
             "revision": source["revision"], "sha256": record["sha256"], "bytes": record["bytes"],
             "license": source["license"], "local_cache": closure["successor_cache_paths"][identifier],
             "generated_code_role": f"gles-successor-{role}", "provenance": source["provenance"]}
    return SourceInput.from_manifest(entry)


def plan() -> CapturePlan:
    contract = CONTRACT.validate()
    producer = contract["producer"]
    if producer["producer_execution_required_for_this_capture"] or producer["producer_execution_proved"]:
        reject("raw GLES capture must not claim producer execution")
    closure = contract["closure"]
    rows = [(closure["root"], "root"), *((row, "core") for row in closure["core_members"]),
            (closure["excluded_extension"], "excluded-extension")]
    if len(rows) != 6 or any(not isinstance(row, dict) for row, _ in rows):
        reject("GLES capture must contain root, four core members, and one exclusion")
    inputs = tuple(source_input(row, role, contract) for row, role in rows)
    if len({item.identifier for item in inputs}) != len(inputs) or len({item.local_cache for item in inputs}) != len(inputs):
        reject("GLES capture has colliding source identities")
    catalog = CONTRACT.COMPOUND.CATALOG["gles-3.2"]
    core, excluded = tuple(catalog["configurations"]), tuple(catalog["excluded_configurations"])
    if len(core) != closure["core_configuration_count"] or len(excluded) != closure["excluded_configuration_count"]:
        reject("GLES configuration count drifted")
    return CapturePlan(contract["contract_sha256"], closure["closure_sha256"],
                       closure["configuration_document_sha256"], inputs, tuple(role for _, role in rows),
                       core, excluded)


def configuration(row: dict[str, object]) -> dict[str, str]:
    selector = row.get("selector")
    parts = PurePosixPath(selector).parts if isinstance(selector, str) else ()
    egl = row.get("use_for_first_egl_config")
    if not parts or selector.startswith("/") or any(part in (".", "..") for part in parts) or not isinstance(egl, bool):
        reject("GLES configuration selector is unsafe")
    value = {"caseListFile": PurePosixPath(selector).name, "commandLine": row.get("command_line"),
             "name": row.get("name"), "os": row.get("os"),
             "useForFirstEGLConfig": "True" if egl else "False"}
    if any(not isinstance(item, str) or not item for item in value.values()):
        reject("GLES configuration has invalid attributes")
    return value


def validate_root(payload: bytes, current: CapturePlan) -> None:
    if not isinstance(payload, bytes) or any(token in payload.upper() for token in (b"<!DOCTYPE", b"<!ENTITY")):
        reject("GLES mustpass root has unsafe XML declarations")
    try:
        root = ET.fromstring(payload)
    except ET.ParseError as error:
        reject(f"GLES mustpass root is invalid XML: {error}")
    if root.tag != "Mustpass" or root.attrib != {"version": "main"} or len(root) != 1:
        reject("GLES mustpass root shape changed")
    package = root[0]
    if package.tag != "TestPackage" or package.attrib != {"name": "Khronos Mustpass ES"}:
        reject("GLES mustpass package changed")
    expected = tuple(configuration(row) for row in (*current.core_configurations, *current.excluded_configurations))
    actual = tuple(node.attrib for node in package if node.tag == "Configuration" and not list(node))
    if len(package) != len(expected) or actual != expected:
        reject("GLES mustpass configurations changed, reordered, or partially captured")
