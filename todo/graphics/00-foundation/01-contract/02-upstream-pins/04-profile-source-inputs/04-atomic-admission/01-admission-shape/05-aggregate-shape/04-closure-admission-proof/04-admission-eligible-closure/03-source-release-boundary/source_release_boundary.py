#!/usr/bin/env python3
"""Fail closed between future source sufficiency and release evidence."""
from __future__ import annotations
import hashlib
import json
import os
import re
import stat
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
RECORD = HERE / "source_release_boundary.json"
REQUIREMENTS = HERE.parents[7] / "03-feature-matrix/01-profile-scope/source_requirements.json"
DOCS_POLICY = HERE.parent / "01-derived-docs-policy/derived_docs_policy.json"
VCTS_HANDOFF = HERE.parents[2] / "05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff.json"
MAX_BYTES = 64 * 1024
DIGEST = re.compile(r"^[0-9a-f]{64}$")
REQUIREMENTS_SHA256 = "a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5"
DOCS_POLICY_SHA256 = "9aac96fcdc340a31e77f44318fd32dbd673eb90551b5ac090d21d8b3a53078aa"
VCTS_HANDOFF_SHA256 = "8b286471e95a957c7d1c9fc9cf41c26d246de86ad1a7cbffb4fb58c3fde682ee"
DOCS_POLICY_DOCUMENT_SHA256 = "86abe34a6c9c9b08707a1b85cc75e9366af5eca1bb90d3f68a86e7eb8d23f4e7"
VCTS_HANDOFF_DOCUMENT_SHA256 = "2df7570fc80b1b02f0f031c3517c05b97baa5a10911f47bfb0f19937712ee0db"
VK_XML = ("xml/vk.xml", "f84d432d5b8912362f96f581f29bbc4f3c8c7843",
          "cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06", 3309653,
          "Apache-2.0 OR MIT (SPDX file notice)", "VK_VERSION_1_4")
ROLES = (("opengl-4.6-core", "api-limit-format-spec", "opengl-46-core-spec"),
         ("opengl-4.6-core", "conformance-manifest", "opengl-cts-manifest"),
         ("gles-3.2", "api-limit-format-spec", "gles-32-spec"),
         ("gles-3.2", "conformance-manifest", "gles-cts-manifest"),
         ("vulkan-1.4-core", "api-limit-format-spec", "vulkan-14-spec"),
         ("vulkan-1.4-core", "conformance-manifest", "vulkan-cts-mustpass"))
CATEGORIES = ("core", "wsi", "video", "extension", "unknown")
RELEASE_EVIDENCE = ("guest-compatibility", "native-reference", "cts-execution",
                    "browser-execution", "performance-measurement")
EFFECTS = frozenset(("source_sufficient", "matrix_may_use_source", "inventory_changed",
                     "f03_changed", "supported", "guest_compatible", "conformant", "certified",
                     "near_native", "release_ready", "admission_eligible", "admitted", "cutover_ready"))
class BoundaryError(ValueError):
    """The boundary cannot safely describe source or release state."""
def reject(message: str) -> None:
    raise BoundaryError(message)
def pairs(items: list[tuple[str, object]]) -> dict[str, object]:
    value: dict[str, object] = {}
    for key, item in items:
        if key in value:
            reject(f"duplicate JSON key {key!r}")
        value[key] = item
    return value
def bytes_at(path: Path, label: str) -> bytes:
    try:
        if not stat.S_ISREG(path.lstat().st_mode):
            reject(f"{label} is not a regular file")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            if not stat.S_ISREG(os.fstat(fd).st_mode):
                reject(f"{label} is not a regular file")
            raw = stream.read(MAX_BYTES + 1)
    except OSError as error:
        reject(f"{label} cannot be read: {error}")
    if len(raw) > MAX_BYTES:
        reject(f"{label} exceeds its bounded size")
    return raw
def decoded(raw: bytes, label: str) -> dict[str, object]:
    try:
        value = json.loads(raw, object_pairs_hook=pairs)
    except (UnicodeDecodeError, json.JSONDecodeError, BoundaryError) as error:
        reject(f"{label} is invalid: {error}")
    if not isinstance(value, dict):
        reject(f"{label} must be an object")
    return value
def document(path: Path, label: str = "boundary") -> dict[str, object]:
    return decoded(bytes_at(path, label), label)
def exact(value: object, fields: frozenset[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or set(value) != fields:
        reject(f"{label} has an invalid schema")
    return value
def fixed(value: dict[str, object], expected: tuple[tuple[str, object, type], ...]) -> bool:
    return all(type(value.get(key)) is kind and value[key] == item for key, item, kind in expected)
def sha(value: object, label: str) -> str:
    if not isinstance(value, str) or not DIGEST.fullmatch(value) or value == "0" * 64:
        reject(f"{label} has an invalid sha256")
    return value
def canonical(value: dict[str, object]) -> bytes:
    return json.dumps({key: item for key, item in value.items() if key != "boundary_sha256"},
                      sort_keys=True, separators=(",", ":")).encode()
def anchored(path: Path, expected: str, label: str) -> dict[str, object]:
    raw = bytes_at(path, label)
    if hashlib.sha256(raw).hexdigest() != expected:
        reject(f"{label} no longer has its reviewed byte identity")
    return decoded(raw, label)
def require_inputs() -> None:
    requirements = anchored(REQUIREMENTS, REQUIREMENTS_SHA256, "F03 source requirements")
    rows = requirements.get("requirements")
    actual = tuple(tuple(row.get(key) for key in ("profile", "role", "required_input_id"))
                   for row in rows if isinstance(row, dict)) if isinstance(rows, list) else ()
    if actual != ROLES:
        reject("F03 source requirements do not bind every reviewed profile role")
    docs = anchored(DOCS_POLICY, DOCS_POLICY_DOCUMENT_SHA256, "derived Docs policy")
    if docs.get("policy_sha256") != DOCS_POLICY_SHA256:
        reject("derived Docs policy self-hash is invalid")
    handoff = anchored(VCTS_HANDOFF, VCTS_HANDOFF_DOCUMENT_SHA256, "V2 canonical-suite handoff")
    if not fixed(handoff, (("handoff_sha256", VCTS_HANDOFF_SHA256, str),
                           ("selector_scope", "khronos-default-mustpass-broader-than-vulkan-1.4-core", str),
                           ("local_filtering", "forbidden", str),
                           ("admitted", False, bool), ("cutover_ready", False, bool),
                           ("satisfies_vulkan_14_core_manifest", False, bool))):
        reject("V2 handoff no longer preserves the broad unadmitted vk-default suite")
def boundary_value(value: object) -> str:
    fields = frozenset(("schema", "contract", "status", "source_requirements_sha256", "source_roles",
                        "future_source_contract", "vulkan_sources", "vk_default_diagnostics",
                        "release_evidence", "prohibitions", "effects", "boundary_sha256"))
    record = exact(value, fields, "source/release boundary")
    if type(record.get("schema")) is not int or (record["schema"], record["contract"], record["status"]) != (
            1, "source-sufficiency-release-boundary-v1", "future-source-contract-required"):
        reject("boundary has an unexpected contract or status")
    require_inputs()
    if sha(record["source_requirements_sha256"], "source_requirements_sha256") != REQUIREMENTS_SHA256:
        reject("boundary does not bind F03 source requirements")
    if type(record["source_roles"]) is not list or tuple(tuple(row) for row in record["source_roles"]
                                                           if type(row) is list) != ROLES or len(record["source_roles"]) != len(ROLES):
        reject("boundary does not require every profile role")
    future = exact(record["future_source_contract"], frozenset(("version", "bind_every_profile_role",
                   "exact_closure_identity_required", "source_sufficient", "f03_transition_allowed")), "future source contract")
    if not fixed(future, (("version", "source-contract-v3", str), ("bind_every_profile_role", True, bool),
                           ("exact_closure_identity_required", True, bool), ("source_sufficient", False, bool),
                           ("f03_transition_allowed", False, bool))):
        reject("boundary weakens future source-contract requirements")
    sources = exact(record["vulkan_sources"], frozenset(("derived_docs_policy_sha256", "vcts_handoff_sha256",
                    "vk_xml", "generated_docs")), "Vulkan sources")
    if sha(sources["derived_docs_policy_sha256"], "derived_docs_policy_sha256") != DOCS_POLICY_SHA256 or sha(sources["vcts_handoff_sha256"], "vcts_handoff_sha256") != VCTS_HANDOFF_SHA256:
        reject("boundary does not bind reviewed Vulkan source identities")
    registry = exact(sources["vk_xml"], frozenset(("selector", "revision", "sha256", "bytes", "license", "version_marker", "classification", "normative_docs_or_cts", "may_satisfy_source_role", "may_satisfy_release_evidence")), "vk.xml")
    generated = exact(sources["generated_docs"], frozenset(("may_satisfy_source", "may_satisfy_release_evidence")), "generated Docs")
    if not fixed(registry, (("selector", VK_XML[0], str), ("revision", VK_XML[1], str), ("sha256", VK_XML[2], str), ("bytes", VK_XML[3], int), ("license", VK_XML[4], str), ("version_marker", VK_XML[5], str), ("classification", "compact-registry-metadata", str), ("normative_docs_or_cts", False, bool), ("may_satisfy_source_role", False, bool), ("may_satisfy_release_evidence", False, bool))) or not fixed(generated, (("may_satisfy_source", False, bool), ("may_satisfy_release_evidence", False, bool))):
        reject("boundary promotes registry metadata or generated Docs")
    suite = exact(record["vk_default_diagnostics"], frozenset(("selector", "scope", "local_filtering",
                  "categories", "categories_change_selector", "reports_are_conformance")), "vk-default diagnostics")
    if not fixed(suite, (("selector", "external/vulkancts/mustpass/main/vk-default.txt", str),
                         ("scope", "khronos-default-mustpass-broader-than-vulkan-1.4-core", str),
                         ("local_filtering", "forbidden", str), ("categories_change_selector", False, bool),
                         ("reports_are_conformance", False, bool))) or suite["categories"] != list(CATEGORIES):
        reject("boundary filters or promotes the broad vk-default diagnostics")
    release = exact(record["release_evidence"], frozenset(("independent_evidence_required", "all_present", "release_claim_permitted")), "release evidence")
    if release["independent_evidence_required"] != list(RELEASE_EVIDENCE) or not fixed(release, (("all_present", False, bool), ("release_claim_permitted", False, bool))):
        reject("boundary permits a release claim without independent evidence")
    if record["prohibitions"] != ["local-core-selector", "category-report-as-conformance", "docs-output-as-source", "docs-output-as-release-evidence", "registry-metadata-as-docs-or-cts", "false-release-flag"]:
        reject("boundary does not retain all substitution prohibitions")
    effects = exact(record["effects"], EFFECTS, "boundary effects")
    if any(item is not False for item in effects.values()):
        reject("boundary effects attempt a source, implementation, or release promotion")
    actual = hashlib.sha256(canonical(record)).hexdigest()
    if sha(record["boundary_sha256"], "boundary_sha256") != actual:
        reject("boundary sha256 does not bind its contents")
    return actual
def boundary(path: Path = RECORD) -> str:
    return boundary_value(document(path))
def main() -> None:
    try:
        if len(sys.argv) not in (1, 2):
            reject("usage: source_release_boundary.py [BOUNDARY.json]")
        value = boundary(Path(sys.argv[1]) if len(sys.argv) == 2 else RECORD)
    except BoundaryError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(2)
    print(f"BOUNDARY: {len(ROLES)} roles; vk-default diagnostic-only; {value}")
if __name__ == "__main__":
    main()
