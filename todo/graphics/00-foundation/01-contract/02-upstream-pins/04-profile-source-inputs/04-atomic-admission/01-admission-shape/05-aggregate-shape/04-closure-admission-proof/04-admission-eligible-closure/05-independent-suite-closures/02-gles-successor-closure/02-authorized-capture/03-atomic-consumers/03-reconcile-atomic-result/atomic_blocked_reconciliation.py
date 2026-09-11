#!/usr/bin/env python3
"""Seal the current atomically blocked Docs, GLES, and VCTS successor state."""
from __future__ import annotations
import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RECORD = HERE / "atomic_blocked_reconciliation.json"
CONSUMERS_DIR = HERE.parent / "02-revalidate-consumers"
CAPTURE_DIR = HERE.parent / "01-bind-sealed-capture"
GLES_DIR, SUITES, ELIGIBLE = HERE.parents[2], HERE.parents[3], HERE.parents[4]
MAX_BYTES, NOFOLLOW = 64 * 1024, getattr(os, "O_NOFOLLOW", 0)
if not NOFOLLOW or not hasattr(os, "O_DIRECTORY"):
    raise RuntimeError("atomic reconciliation requires no-follow directory descriptors")
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)
FILE_FLAGS = os.O_RDONLY | NOFOLLOW | os.O_NONBLOCK | getattr(os, "O_CLOEXEC", 0)
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "admission_eligible",
           "admitted", "cutover_ready", "supported", "conformant", "certified", "near_native",
           "performance_claimed", "satisfies_vulkan_14_core_manifest")
MISSING = ("opengl-46-core-spec", "opengl-cts-manifest", "gles-32-spec", "gles-cts-manifest",
           "vulkan-14-spec", "vulkan-cts-mustpass")
CLOSURE = "fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4"
GLES_ROOT = "9f466f19a26120149bab06e70596c9fff57b61d767cfdbadc7ba13934ce2ea96"
F02_LOCK = "08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6"
SEMANTIC = {"binding": "42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7",
            "consumer": "d858ec3957a989da9b471b925ba6f5d4445f910a810ab3c3d897945394aa80c6",
            "integration": "d22eeadf9da86ccd79493e348d7809d090d01dbf91c9a4e4316b6db92d2caf7c",
            "docs": "d9317d418f6c13f4c85275bffb1a01139eb036e7ddf5853a347dde84d69273c5",
            "vcts": "8b286471e95a957c7d1c9fc9cf41c26d246de86ad1a7cbffb4fb58c3fde682ee",
            "boundary": "da10dbc51230b2495f7c24c100055ab91970a92217b43f2895ec5cfa2ead459b",
            "receipt": "9ff372bcfeb83538882da396b2ecbb0801703ca55a1b0dd38fa7ab51d4bf501a"}
RAW = {"binding": "f765b917f8144288205fad737d68e5efd02e0665f0f8e140f0ca138ad32740aa",
       "consumer": "4f1f3ff488c95f8cdfe42ec0d4c1f07dbc91cd39d8cfed9d8529ac39e93326dc",
       "integration": "9283bc20bbba922707563625b5dbc7162881edf7c883e9a876afc8c00c98bd82",
       "docs": "492b900c82d40d2544b7f619f26e333d33eb79122cbef1d4502b8406ac5a09d3",
       "vcts": "2df7570fc80b1b02f0f031c3517c05b97baa5a10911f47bfb0f19937712ee0db",
       "boundary": "6a0a7fc4562acb63678ccfc2d6c965355bcce7b417a0483734119c84698025ff",
       "receipt": "1b15139349b59c05830707a5b2283be4dd5db823231ce874e7c4b436c4b72660"}

class AtomicError(ValueError):
    """An aggregate is partial, stale, mixed, or falsely promoted."""
def reject(message: str) -> None: raise AtomicError(message)
def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None: raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec); sys.modules[name] = module
    try: spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(name, None); raise
    return module

BINDING = reviewed("f025_atomic_binding", CAPTURE_DIR / "sealed_capture_binding.py")
CONSUMERS = reviewed("f025_atomic_consumers", CONSUMERS_DIR / "consumer_revalidation.py")
INTEGRATION = reviewed("f025_atomic_integration", GLES_DIR / "01-successor-integration/multi_suite_successor_integration.py")
DOCS = reviewed("f025_atomic_docs", ELIGIBLE / "06-successor-inventory-transition/successor_inventory_transition.py")
VCTS = reviewed("f025_atomic_vcts", HERE.parents[6] / "05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.py")
BOUNDARY = reviewed("f025_atomic_boundary", SUITES / "01-successor-boundary/suite_successor_boundary.py")
RECEIPT = reviewed("f025_atomic_receipt", HERE.parents[5] / "03-blocked-state-receipt/blocked_state_receipt.py")
PATHS = {"binding": BINDING.RECORD, "consumer": CONSUMERS.RECORD, "integration": INTEGRATION.RECORD,
         "docs": DOCS.RECORD, "vcts": VCTS.HANDOFF, "boundary": BOUNDARY.RECORD, "receipt": RECEIPT.RECEIPT}

def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value: reject("aggregate has a duplicate JSON key")
        value[key] = item
    return value
def stamp(info: os.stat_result) -> tuple[int, int, int, int, int]:
    return info.st_dev, info.st_ino, info.st_size, info.st_ctime_ns, info.st_nlink
def opened(path: Path) -> int:
    absolute = Path(os.path.abspath(path))
    try:
        current = os.open(absolute.anchor, DIR_FLAGS)
        try:
            for part in absolute.parts[1:-1]:
                successor = os.open(part, DIR_FLAGS, dir_fd=current); os.close(current); current = successor
            named = os.stat(absolute.name, dir_fd=current, follow_symlinks=False)
            descriptor = os.open(absolute.name, FILE_FLAGS, dir_fd=current)
            if stamp(named) != stamp(os.fstat(descriptor)): os.close(descriptor); reject("aggregate input changed before safe open")
            return descriptor
        finally: os.close(current)
    except OSError as error: reject(f"aggregate input cannot be safely opened: {error}")
def raw(path: Path) -> bytes:
    descriptor = opened(path)
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > MAX_BYTES: reject("aggregate input is not a bounded regular file")
        chunks, remaining = [], before.st_size + 1
        while remaining:
            chunk = os.read(descriptor, min(8192, remaining))
            if not chunk: break
            chunks.append(chunk); remaining -= len(chunk)
        value, after = b"".join(chunks), os.fstat(descriptor)
    finally: os.close(descriptor)
    if stamp(before) != stamp(after) or len(value) != before.st_size or len(value) > MAX_BYTES: reject("aggregate input changed during bounded read")
    return value
def document(path: Path) -> dict[str, object]:
    try: value = json.loads(raw(path).decode("utf-8"), object_pairs_hook=pairs, parse_constant=lambda text: reject(f"aggregate has non-finite {text}"))
    except AtomicError: raise
    except (UnicodeDecodeError, ValueError) as error: reject(f"aggregate is invalid JSON: {error}")
    if not isinstance(value, dict): reject("aggregate is not an object")
    return value
def snapshot() -> dict[str, str]: return {name: hashlib.sha256(raw(path)).hexdigest() for name, path in PATHS.items()}
def digest(value: dict[str, object]) -> str:
    return hashlib.sha256(json.dumps({key: item for key, item in value.items() if key != "reconciliation_sha256"}, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
def alike(left: object, right: object) -> bool:
    if type(left) is not type(right): return False
    if isinstance(left, dict): return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list): return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    if isinstance(left, tuple): return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right
def false(value: object) -> bool:
    return isinstance(value, dict) and bool(value) and all(type(item) is bool and item is False for item in value.values())

def inputs():
    before = snapshot()
    try: values = (BINDING.validate(), CONSUMERS.validate(), INTEGRATION.validate(), DOCS.transition(), VCTS.validate(), BOUNDARY.validate(), RECEIPT.validate())
    except Exception as error: reject(f"atomic predecessor is invalid: {error}")
    if before != snapshot() or before != RAW: reject("atomic predecessor documents changed or have stale raw identities")
    return values
def build() -> dict[str, object]:
    binding, consumer, integration, transition, handoff, boundary, receipt = inputs()
    docs = document(DOCS.RECORD); capture, f02, f03 = binding["capture"], consumer["active_f02"], consumer["f03"]
    try:
        semantic = (binding["binding_sha256"], consumer["consumer_revalidation_sha256"], integration["integration_sha256"], transition, handoff["handoff_sha256"], boundary["policy_sha256"], receipt["receipt_sha256"])
        graph = (consumer["binding"]["binding_sha256"], consumer["binding"]["binding_document_sha256"], consumer["binding"]["closure_sha256"], binding["binding_sha256"], RAW["binding"], capture["closure_sha256"], binding["integration_sha256"], integration["integration_sha256"], integration["docs"]["transition_sha256"], transition, integration["gles"]["boundary_policy_sha256"], boundary["policy_sha256"], boundary["vcts"]["handoff_sha256"], handoff["handoff_sha256"], integration["predecessor"]["inventory_lock_sha256"], f02["inventory_lock_sha256"], docs["predecessor"]["lock_sha256"])
        rules = (binding["status"], consumer["status"], integration["status"], docs["status"], handoff["status"], boundary["status"], integration["aggregate"]["wrapper_families"], integration["aggregate"]["cross_wrapper_substitution_allowed"], integration["aggregate"]["active_family_alias_allowed"], consumer["binding"]["active_alias_allowed"], f02["captured_member_ids_present"], f02["captured_member_fingerprints_present"], f03["profile_status"], f03["blocker"], tuple(f03["missing_required_input_ids"]), docs["cutover"]["active_mutation_permitted_by_this_record"], docs["cutover"]["f03_transition_allowed"], docs["successor"]["raw_member_max_bytes"], docs["successor"]["generated_outputs_are_raw_sources"], docs["generated"]["output_can_be_raw_source"], docs["successor"]["legacy_vulkan_alias_allowed"], handoff["member_count"], handoff["member_total_bytes"], handoff["selector_scope"], handoff["local_filtering"], handoff["docs_generated_artifacts"], handoff["admitted"], handoff["cutover_ready"], handoff["satisfies_vulkan_14_core_manifest"], boundary["vcts"]["f02_source_member_cap_bytes"], boundary["vcts"]["over_f02_cap_member_count"], boundary["vcts"]["local_selector_allowed"], boundary["vcts"]["taxonomy_is_conformance"], boundary["vcts"]["core_manifest_ready"], integration["gles"]["family"], boundary["gles"]["candidate_id"], boundary["gles"]["candidate_sha256"], capture["members"][0]["id"], capture["members"][0]["sha256"], capture["members"][0]["role"])
    except (KeyError, IndexError, TypeError) as error: reject(f"atomic predecessor shape is incomplete: {error}")
    expected_rules = ("captured-unadmitted", "blocked-unchanged-consumers", "design-only-unadmitted", "design-only-unadmitted", "verified-canonical-suite-receipt-unadmitted", "policy-only-unadmitted", ["vulkan-docs", "gles-cts"], False, False, False, False, False, "blocked", "inventory-sources-incomplete", MISSING, False, False, 8 * 1024 * 1024, False, False, False, 98, 434669348, "khronos-default-mustpass-broader-than-vulkan-1.4-core", "forbidden", "outside-admitted-implementation-source-closure", False, False, False, 8 * 1024 * 1024, 14, False, False, False, "gles-cts", "gles-cts-manifest", GLES_ROOT, "gles-cts-manifest", GLES_ROOT, "root")
    expected_graph = (SEMANTIC["binding"], RAW["binding"], CLOSURE, SEMANTIC["binding"], RAW["binding"], CLOSURE, SEMANTIC["integration"], SEMANTIC["integration"], SEMANTIC["docs"], SEMANTIC["docs"], SEMANTIC["boundary"], SEMANTIC["boundary"], SEMANTIC["vcts"], SEMANTIC["vcts"], F02_LOCK, F02_LOCK, F02_LOCK)
    if not alike(semantic, tuple(SEMANTIC.values())) or not alike(graph, expected_graph) or not alike(rules, expected_rules) or not all(false(value) for value in (binding["readiness"], consumer["effects"], consumer["historical_receipt"]["readiness"], integration["effects"], docs["effects"], boundary["effects"])):
        reject("atomic predecessors do not retain the exact blocked boundary")
    value: dict[str, object] = {"schema": 1, "kind": "atomic-blocked-successor-reconciliation-v1", "status": "atomically-blocked-unadmitted", "documents": {name: {"semantic_sha256": SEMANTIC[name], "document_sha256": RAW[name]} for name in RAW if name != "receipt"}, "wrappers": {"families": ["vulkan-docs", "gles-cts"], "cross_wrapper_substitution_allowed": False, "active_alias_allowed": False}, "gles": {"binding_sha256": SEMANTIC["binding"], "consumer_sha256": SEMANTIC["consumer"], "closure_sha256": capture["closure_sha256"], "f02_f03_unchanged": True}, "docs": {"transition_sha256": SEMANTIC["docs"], "raw_member_max_bytes": 8 * 1024 * 1024, "independently_admitted": False, "active_or_f03_mutation": False}, "vcts": {"handoff_sha256": SEMANTIC["vcts"], "member_count": 98, "member_total_bytes": 434669348, "selector_scope": handoff["selector_scope"], "local_filtering": "forbidden", "core_manifest_ready": False}, "historical_receipt": {"role": "historical-only", "receipt_sha256": SEMANTIC["receipt"], "document_sha256": RAW["receipt"], "first_blocker": ["gles-cts-manifest", "requires-multifile-core-selector-closure"]}, "f03": {"status": "blocked", "missing_required_input_ids": list(MISSING)}, "global_blockers": [["vulkan-14-spec", "vulkan-docs-core-generated-closure-unadmitted"], ["vulkan-cts-mustpass", "vcts-vk-default-compound-oversize-core-scope-unadmitted"]], "effects": {key: False for key in EFFECTS}, "reconciliation_sha256": ""}
    value["reconciliation_sha256"] = digest(value); return value
def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    if not isinstance(value.get("reconciliation_sha256"), str) or value["reconciliation_sha256"] != digest(value): reject("atomic reconciliation self-hash is invalid")
    if not alike(value, build()): reject("atomic reconciliation does not bind the exact blocked aggregate")
    return value
def main() -> None:
    try:
        value = build() if sys.argv[1:] == ["--build"] else validate() if len(sys.argv) == 1 else None
        if value is None: raise SystemExit("usage: atomic_blocked_reconciliation.py [--build]")
    except AtomicError as error:
        print(f"FAIL: {error}", file=sys.stderr); raise SystemExit(2)
    print(json.dumps(value, indent=2, sort_keys=True) if sys.argv[1:] else f"ATOMIC: {value['status']} {value['reconciliation_sha256']}")
if __name__ == "__main__": main()
