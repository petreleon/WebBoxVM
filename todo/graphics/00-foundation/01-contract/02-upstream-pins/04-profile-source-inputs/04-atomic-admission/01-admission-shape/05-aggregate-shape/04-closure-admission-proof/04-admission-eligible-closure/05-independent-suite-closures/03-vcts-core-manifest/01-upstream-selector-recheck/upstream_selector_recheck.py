#!/usr/bin/env python3
"""Seal an offline observation that Khronos has not published a VCTS core selector."""
from __future__ import annotations
import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECORD = HERE / "upstream_selector_recheck.json"
V2_PATH = HERE.parents[4] / "05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py"
BOUNDARY_PATH = HERE.parents[1] / "01-successor-boundary/suite_successor_boundary.py"
RELEASE_PATH = HERE.parents[2] / "03-source-release-boundary/source_release_boundary.py"
MAX_BYTES, NOFOLLOW = 64 * 1024, getattr(os, "O_NOFOLLOW", 0)
BLOCKER = "missing-khronos-published-immutable-explicit-vulkan-1.4-core-vcts-manifest"
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "source_sufficient",
           "matrix_may_use_source", "admission_eligible", "admitted", "cutover_ready", "supported",
           "conformant", "certified", "near_native", "performance_claimed", "satisfies_vulkan_14_core_manifest")
FIELDS = frozenset(("schema", "kind", "status", "observation", "predecessors", "prohibitions",
                    "blocker", "effects", "recheck_sha256"))
EXPECTED_HANDOFF = ("verified-canonical-suite-receipt-unadmitted",
                    "30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218", 98, 434669348,
                    "khronos-default-mustpass-broader-than-vulkan-1.4-core", "forbidden", False, False, False)
EXPECTED_BOUNDARY = ({"core": 0, "wsi": 1, "video": 1, "extension": 4, "unknown": 92}, 14, False, False, False)
RELEASE_SHA256 = "7cc44e773bb45e61535f535c93640547dfddead6b0b2f0bb23cc73c62ec06485"
EXPECTED_RAW = {"v2_root_identity": "c2fc1ee3bb8113f71c50b9da94409aba6a7dd96cd59e3a3eed508b40f701c5b3",
                "v2_handoff": "2df7570fc80b1b02f0f031c3517c05b97baa5a10911f47bfb0f19937712ee0db",
                "successor_boundary": "6a0a7fc4562acb63678ccfc2d6c965355bcce7b417a0483734119c84698025ff",
                "source_release_boundary": "4544bc95c6021fd85d134eea5d4f2913d4fdaf810e9058d74d220dda7d1cdc40",
                "active_f02_inventory_lock": "08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6",
                "active_f03_requirements": "a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5"}

class RecheckError(ValueError):
    """The observation is malformed, stale, mixed, or promotes unavailable authority."""
def reject(message: str) -> None: raise RecheckError(message)
def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec); sys.modules[name] = module
    try: spec.loader.exec_module(module)
    except Exception: sys.modules.pop(name, None); raise
    return module

if not NOFOLLOW: raise RuntimeError("recheck requires no-follow file descriptors")
V2 = reviewed("f025_vcts_recheck_v2", V2_PATH)
BOUNDARY = reviewed("f025_vcts_recheck_boundary", BOUNDARY_PATH)
RELEASE = reviewed("f025_vcts_recheck_release", RELEASE_PATH)
INPUTS = {"v2_root_identity": V2.IDENTITY, "v2_handoff": V2.HANDOFF,
          "successor_boundary": BOUNDARY.RECORD, "source_release_boundary": RELEASE.RECORD,
          "active_f02_inventory_lock": HERE.parents[8] / "01-input-inventory/inventory.lock",
          "active_f03_requirements": RELEASE.REQUIREMENTS}

def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value: reject("JSON has a duplicate key")
        value[key] = item
    return value
def stamp(info: os.stat_result) -> tuple[int, int, int, int, int]:
    return info.st_dev, info.st_ino, info.st_size, info.st_ctime_ns, info.st_nlink
def raw(path: Path, label: str, limit: int = MAX_BYTES) -> bytes:
    try:
        named = path.lstat()
        if not stat.S_ISREG(named.st_mode): reject(f"{label} is not a regular file")
        descriptor = os.open(path, os.O_RDONLY | NOFOLLOW | os.O_NONBLOCK)
        try:
            opened = os.fstat(descriptor)
            if not stat.S_ISREG(opened.st_mode) or opened.st_nlink != 1 or stamp(named) != stamp(opened):
                reject(f"{label} changed before safe open")
            value = os.read(descriptor, limit + 1)
            while len(value) <= limit and len(value) < opened.st_size:
                chunk = os.read(descriptor, limit + 1 - len(value))
                if not chunk: break
                value += chunk
            if stamp(opened) != stamp(os.fstat(descriptor)): reject(f"{label} changed during read")
        finally: os.close(descriptor)
    except OSError as error: reject(f"{label} cannot be read: {error}")
    if len(value) > limit or len(value) != opened.st_size: reject(f"{label} exceeds its bounded size or changed during read")
    return value
def document(path: Path, label: str) -> dict[str, object]:
    try:
        value = json.loads(raw(path, label).decode("utf-8"), object_pairs_hook=pairs,
                           parse_constant=lambda text: reject(f"JSON has non-finite {text}"))
    except RecheckError: raise
    except (UnicodeDecodeError, json.JSONDecodeError) as error: reject(f"{label} cannot be parsed: {error}")
    if not isinstance(value, dict): reject(f"{label} is not an object")
    return value
def digest(value: dict[str, object]) -> str:
    return hashlib.sha256(json.dumps({key: item for key, item in value.items() if key != "recheck_sha256"}, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
def alike(left: object, right: object) -> bool:
    if type(left) is not type(right): return False
    if isinstance(left, dict): return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list): return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right
def hashes() -> dict[str, str]: return {name: hashlib.sha256(raw(path, name)).hexdigest() for name, path in INPUTS.items()}
def false(value: object, keys: tuple[str, ...] | None = None) -> bool:
    return isinstance(value, dict) and (keys is None or set(value) == set(keys)) and bool(value) and all(type(item) is bool and item is False for item in value.values())

def predecessors() -> dict[str, object]:
    before = hashes()
    try:
        handoff, boundary, release_sha256 = V2.validate(), BOUNDARY.validate(), RELEASE.boundary()
        release = document(RELEASE.RECORD, "source/release boundary")
    except Exception as error: reject(f"reviewed predecessor is invalid: {error}")
    if before != EXPECTED_RAW or before != hashes(): reject("reviewed predecessor changed or has stale raw identities")
    vcts, future = boundary["vcts"], release["future_source_contract"]
    handoff_state = tuple(handoff[key] for key in ("status", "root_identity_sha256", "member_count", "member_total_bytes", "selector_scope", "local_filtering", "admitted", "cutover_ready", "satisfies_vulkan_14_core_manifest"))
    boundary_state = (vcts["category_counts"], vcts["over_f02_cap_member_count"], vcts["local_selector_allowed"], vcts["taxonomy_is_conformance"], vcts["core_manifest_ready"])
    role = ["vulkan-1.4-core", "conformance-manifest", "vulkan-cts-mustpass"]
    if (not alike(handoff_state, EXPECTED_HANDOFF) or not alike(boundary_state, EXPECTED_BOUNDARY)
            or not false(boundary["effects"], BOUNDARY.EFFECTS) or role not in release["source_roles"]
            or (future["source_sufficient"], future["f03_transition_allowed"], release_sha256) != (False, False, RELEASE_SHA256)
            or not false(release["effects"], tuple(release["effects"]))):
        reject("reviewed predecessors do not retain the exact blocked VCTS state")
    return {"v2_root_identity": {"semantic_sha256": handoff["root_identity_sha256"], "document_sha256": before["v2_root_identity"]},
            "v2_handoff": {"semantic_sha256": handoff["handoff_sha256"], "document_sha256": before["v2_handoff"]},
            "successor_boundary": {"semantic_sha256": boundary["policy_sha256"], "document_sha256": before["successor_boundary"]},
            "source_release_boundary": {"semantic_sha256": release_sha256, "document_sha256": before["source_release_boundary"]},
            "active_f02": {"inventory_lock_document_sha256": before["active_f02_inventory_lock"], "changed": False},
            "active_f03": {"source_requirements_document_sha256": before["active_f03_requirements"], "changed": False}}

def build() -> dict[str, object]:
    value: dict[str, object] = {
        "schema": 1, "kind": "vcts-upstream-selector-recheck-v1", "status": "blocked-upstream-authority-missing",
        "observation": {"observed_on": "2026-09-11", "repository": "KhronosGroup/VK-GL-CTS", "latest_vulkan_14_tag_at_observation": "vulkan-cts-1.4.6.2",
                        "tag_object_git_oid": "42c723aa10d2652590f02741827aef43b0421d23", "peeled_commit_git_oid": "f6a29701220f34dd1407513bfe80d74ca7b392ce",
                        "release_family_tags": ["vulkan-cts-1.4.6.2", "vulkan-cts-1.4.6.1", "vulkan-cts-1.4.6.0"],
                        "root_directory_files": ["vk-default.txt", "vk-fraction-mandatory-tests.txt", "vksc-default.txt"],
                        "vk_default": {"path": "external/vulkancts/mustpass/main/vk-default.txt", "bytes": 3347, "sha256": "b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4", "includes_wsi": True, "includes_video": True, "scope": "khronos-default-mustpass-broader-than-vulkan-1.4-core"},
                        "explicit_vulkan_14_core_manifest_present": False},
        "predecessors": predecessors(),
        "prohibitions": {"local_core_selector_allowed": False, "taxonomy_as_conformance_allowed": False, "root_only_claim_allowed": False, "opaque_member_splitting_allowed": False, "vcts_as_docs_allowed": False, "tag_alone_sufficient": False},
        "blocker": {"id": BLOCKER, "replaces_historical_global_blocker": False},
        "effects": {name: False for name in EFFECTS}, "recheck_sha256": ""}
    value["recheck_sha256"] = digest(value)
    return value
def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path, "recheck")
    if set(value) != FIELDS or type(value.get("schema")) is not int: reject("recheck has an unexpected schema")
    if not isinstance(value.get("recheck_sha256"), str) or value["recheck_sha256"] != digest(value): reject("recheck self-hash is invalid")
    if not alike(value, build()): reject("recheck does not bind the exact blocked upstream observation")
    return value
def main() -> None:
    try:
        value = build() if sys.argv[1:] == ["--build"] else validate() if len(sys.argv) == 1 else None
        if value is None: raise SystemExit("usage: upstream_selector_recheck.py [--build]")
    except RecheckError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)
    print(json.dumps(value, indent=2, sort_keys=True) if sys.argv[1:] else f"BLOCKED: {BLOCKER} {value['recheck_sha256']}")
if __name__ == "__main__": main()
