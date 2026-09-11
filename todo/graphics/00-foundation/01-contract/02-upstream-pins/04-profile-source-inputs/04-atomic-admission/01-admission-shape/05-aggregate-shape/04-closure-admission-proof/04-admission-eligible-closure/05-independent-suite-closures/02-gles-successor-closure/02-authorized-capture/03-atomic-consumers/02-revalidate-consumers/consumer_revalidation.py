#!/usr/bin/env python3
"""Revalidate unchanged F02/F03 consumers against one sealed GLES capture."""
from __future__ import annotations
import hashlib, importlib.util, json, os, stat, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
RECORD = HERE / "consumer_revalidation.json"
UPSTREAM, PROOF, CONTRACT = HERE.parents[10], HERE.parents[5], HERE.parents[11]
INVENTORY_DIR = UPSTREAM / "01-input-inventory"
MANIFEST, LOCK = INVENTORY_DIR / "manifest.toml", INVENTORY_DIR / "inventory.lock"
PARTS = tuple(INVENTORY_DIR / f"inputs/part-{number:04d}.toml" for number in (1, 2))
MAX_BYTES = 32 * 1024
NOFOLLOW = getattr(os, "O_NOFOLLOW", 0)
if not NOFOLLOW or not hasattr(os, "O_DIRECTORY"): raise RuntimeError("consumer revalidation requires no-follow directory descriptors")
DIR_FLAGS = os.O_RDONLY | os.O_DIRECTORY | NOFOLLOW | getattr(os, "O_CLOEXEC", 0)
FILE_FLAGS = os.O_RDONLY | NOFOLLOW | os.O_NONBLOCK | getattr(os, "O_CLOEXEC", 0)
MISSING = ("opengl-46-core-spec", "opengl-cts-manifest", "gles-32-spec", "gles-cts-manifest",
           "vulkan-14-spec", "vulkan-cts-mustpass")
EFFECTS = ("active_inventory_changed", "cache_freshness_proved", "f03_changed", "admission_eligible",
           "admitted", "cutover_ready", "supported", "conformant", "certified", "near_native",
           "performance_claimed", "satisfies_vulkan_14_core_manifest")
class ConsumerError(ValueError):
    """A consumer revalidation is stale, aliased, or falsely promoting."""
def reject(message: str) -> None:
    raise ConsumerError(message)
def reviewed(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load reviewed module {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module
BINDING = reviewed("f025_consumer_binding", HERE.parent / "01-bind-sealed-capture/sealed_capture_binding.py")
LAYOUT = reviewed("f025_consumer_inventory", INVENTORY_DIR / "inventory_layout.py")
RECEIPT = reviewed("f025_consumer_receipt", PROOF / "03-blocked-state-receipt/blocked_state_receipt.py")
F03 = reviewed("f025_consumer_f03", CONTRACT / "03-feature-matrix/01-profile-scope/profile_contract.py")
def pairs(rows: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in rows:
        if key in value:
            reject("consumer receipt has a duplicate JSON key")
        value[key] = item
    return value
def identity(info: os.stat_result) -> tuple[int, int, int, int, int]:
    return info.st_dev, info.st_ino, info.st_size, info.st_ctime_ns, info.st_nlink
def opened(path: Path) -> int:
    absolute = Path(os.path.abspath(path))
    try:
        current = os.open(absolute.anchor, DIR_FLAGS)
        try:
            for part in absolute.parts[1:-1]:
                successor = os.open(part, DIR_FLAGS, dir_fd=current)
                os.close(current); current = successor
            named = os.stat(absolute.name, dir_fd=current, follow_symlinks=False)
            descriptor = os.open(absolute.name, FILE_FLAGS, dir_fd=current)
            if identity(named) != identity(os.fstat(descriptor)):
                os.close(descriptor); reject("consumer input changed before safe open")
            return descriptor
        finally:
            os.close(current)
    except OSError as error:
        reject(f"consumer input cannot be safely opened: {error}")
def raw(path: Path) -> bytes:
    descriptor = opened(path)
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > MAX_BYTES:
            reject("consumer input is not a bounded regular file")
        chunks, remaining = [], before.st_size + 1
        while remaining:
            chunk = os.read(descriptor, min(8192, remaining))
            if not chunk:
                break
            chunks.append(chunk); remaining -= len(chunk)
        value, after = b"".join(chunks), os.fstat(descriptor)
    finally:
        os.close(descriptor)
    if identity(before) != identity(after) or len(value) != before.st_size or len(value) > MAX_BYTES:
        reject("consumer input changed during bounded read")
    return value
def document(path: Path) -> dict[str, object]:
    try:
        value = json.loads(raw(path).decode("utf-8"), object_pairs_hook=pairs,
                           parse_constant=lambda value: reject(f"consumer receipt has non-finite {value}"))
    except ConsumerError:
        raise
    except (UnicodeDecodeError, ValueError) as error:
        reject(f"consumer receipt is invalid JSON: {error}")
    if not isinstance(value, dict):
        reject("consumer receipt is not an object")
    return value
def sha256(path: Path) -> str:
    return hashlib.sha256(raw(path)).hexdigest()
def digest(value: dict[str, object]) -> str:
    body = {key: item for key, item in value.items() if key != "consumer_revalidation_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
def alike(left: object, right: object) -> bool:
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return set(left) == set(right) and all(alike(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(alike(one, two) for one, two in zip(left, right))
    return left == right
def build() -> dict[str, object]:
    try:
        binding, inventory, receipt, missing = BINDING.validate(), LAYOUT.load_inventory(MANIFEST), RECEIPT.validate(), F03.validate()
    except Exception as error:
        reject(f"consumer predecessor is invalid: {error}")
    capture = binding.get("capture")
    members = capture.get("members") if isinstance(capture, dict) else None
    captured = tuple(item.get("id") for item in members) if isinstance(members, list) and all(isinstance(item, dict) for item in members) else ()
    fingerprints = tuple((item.get("sha256"), item.get("bytes")) for item in members) if isinstance(members, list) and all(isinstance(item, dict) for item in members) else ()
    active = tuple(str(item["id"]) for item in inventory.inputs)
    active_fingerprints = tuple((item["sha256"], item["bytes"]) for item in inventory.inputs)
    lock_sha, manifest_sha, part_sha = sha256(LOCK), sha256(MANIFEST), tuple(sha256(path) for path in PARTS)
    scope = F03.document(F03.SCOPE)
    profiles = scope.get("profiles")
    profile_rows = tuple((row.get("id"), row.get("status"), row.get("blocker")) for row in profiles) if isinstance(profiles, list) and all(isinstance(row, dict) for row in profiles) else ()
    if (binding.get("binding_sha256"), sha256(BINDING.RECORD), binding.get("status"), capture.get("closure_sha256") if isinstance(capture, dict) else None,
            inventory.schema, inventory.revision, lock_sha, manifest_sha, part_sha, len(active),
            receipt.get("receipt_sha256"), sha256(RECEIPT.RECEIPT), receipt.get("state"), tuple(receipt.get("readiness", {}).values()),
            sha256(F03.REQUIREMENTS_PATH), sha256(F03.SCOPE), tuple(missing), profile_rows,
            (bool(set(captured) & set(active)), bool(set(fingerprints) & set(active_fingerprints)))) != (
            "42a221ba44ad7f44aefc40fe876933000b452284d659db00bed5d34da6a525e7", "f765b917f8144288205fad737d68e5efd02e0665f0f8e140f0ca138ad32740aa", "captured-unadmitted", "fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4",
            2, "08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6", "08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6", "f6b4b8bc18751d4645cf638faa4958b9ddb01c5e12815ad599f7eb7ce26c4adb", ("eef0b188e9e663e6be483bfedde5df27687db9b590ff2bd208ebf462b0cc6649", "c58855fd366ddd8d8b8b135f125d6f0b860bd79eb8f0d0576e937d86df656ad0"), 17,
            "9ff372bcfeb83538882da396b2ecbb0801703ca55a1b0dd38fa7ab51d4bf501a", "1b15139349b59c05830707a5b2283be4dd5db823231ce874e7c4b436c4b72660", "blocked", (False,) * 5,
            "a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5", "15fc0abd0a174db43fc6a46f17a5ae06e3376eaed0325a49b8a6e874eb1767d4", MISSING,
            (("opengl-4.6-core", "blocked", "inventory-sources-incomplete"), ("gles-3.2", "blocked", "inventory-sources-incomplete"), ("vulkan-1.4-core", "blocked", "inventory-sources-incomplete")), (False, False)):
        reject("consumers do not retain the exact blocked F02/F03 boundary")
    value: dict[str, object] = {
        "schema": 1, "kind": "f02-f03-consumer-revalidation-v1", "status": "blocked-unchanged-consumers",
        "binding": {"binding_sha256": binding["binding_sha256"], "binding_document_sha256": sha256(BINDING.RECORD),
                    "closure_sha256": capture["closure_sha256"], "status": binding["status"], "active_alias_allowed": False},
        "active_f02": {"schema": inventory.schema, "inventory_lock_sha256": lock_sha, "manifest_sha256": manifest_sha,
                       "part_sha256": list(part_sha), "input_count": len(active), "captured_member_ids": list(captured),
                       "captured_member_ids_present": False, "captured_member_fingerprints_present": False},
        "historical_receipt": {"receipt_sha256": receipt["receipt_sha256"], "document_sha256": sha256(RECEIPT.RECEIPT),
                               "state": receipt["state"], "readiness": receipt["readiness"]},
        "f03": {"requirements_sha256": sha256(F03.REQUIREMENTS_PATH), "scope_sha256": sha256(F03.SCOPE),
                "missing_required_input_ids": list(missing), "profile_status": "blocked", "blocker": "inventory-sources-incomplete"},
        "effects": {key: False for key in EFFECTS}, "consumer_revalidation_sha256": ""}
    value["consumer_revalidation_sha256"] = digest(value)
    return value
def validate(path: Path = RECORD) -> dict[str, object]:
    value = document(path)
    if not isinstance(value.get("consumer_revalidation_sha256"), str) or value["consumer_revalidation_sha256"] != digest(value):
        reject("consumer receipt self-hash is invalid")
    if not alike(value, build()):
        reject("consumer receipt does not bind exact blocked consumers")
    return value
def main() -> None:
    try:
        value = build() if sys.argv[1:] == ["--build"] else validate() if len(sys.argv) == 1 else None
        if value is None:
            raise SystemExit("usage: consumer_revalidation.py [--build]")
    except ConsumerError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(json.dumps(value, indent=2, sort_keys=True) if sys.argv[1:] else f"CONSUMERS: {value['status']} {value['consumer_revalidation_sha256']}")
if __name__ == "__main__":
    main()
