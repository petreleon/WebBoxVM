"""Exact raw GLES full-root member records, separate from the local observation."""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "01-canonical-full-suite-roots"))
import full_suite_roots as roots  # noqa: E402

ROOT_ID, ROOT_PROFILE = "gles-cts-main", "gles-3.2"
MEMBERS = (
    ("gles2-khr-main.txt", "aed17d34d64047973c439835ee7cfa8a109c2159512fe1fe59fb07fc05d395f2", 38127, 473),
    ("gles3-khr-main.txt", "c79097f9f9c69a4556e235b3301c0dc9a5d73375c05f193a6f5e127d853ace72", 443126, 6498),
    ("gles31-khr-main.txt", "9fd8e4ff51616262c567f7a9eec83f69d2d311a4293a45d2d377306d426d520e", 288777, 4101),
    ("gles32-khr-main.txt", "426fa31d557e08e214b75fda5d9efcaec54a42cdc14d4915939a7d02ccb6f249", 105294, 1405),
    ("gles32-khr-glesext.txt", "789b0474c61693baaa7cbc941575f23b383e2ba3c4991b1a910e046a17a6e0e6", 81374, 1097),
)
NO_CLAIMS = {name: False for name in roots.CLAIMS}


class RecordError(ValueError):
    """A raw GLES full-root member record is not the reviewed exact closure."""


def reject(message: str) -> None:
    raise RecordError(message)


def root_record() -> dict[str, object]:
    value = roots.catalog()
    roots.validate_full_suite_catalog(value)
    record = next((item for item in value["records"] if item["id"] == ROOT_ID), None)
    if record is None or record["profile"] != ROOT_PROFILE:
        reject("GLES ledger has no canonical full-suite root")
    return record


def member_id(name: str) -> str:
    return f"gles-cts-{name.removesuffix('.txt')}"


def member_records(root: dict[str, object]) -> list[dict[str, object]]:
    base, url = root["selector_path"].rsplit("/", 1)[0], root["immutable_url"].rsplit("/", 1)[0]
    result = []
    for name, digest, size, _cases in MEMBERS:
        identifier, path = member_id(name), f"{base}/{name}"
        result.append({"kind": "upstream-source", "id": identifier, "sha256": digest, "bytes": size,
                       "license": root["license"], "attribution": root["attribution"], "scope": "suite-member",
                       "authority": "Khronos", "producer": "Khronos", "claims": dict(NO_CLAIMS),
                       "immutable_url": f"{url}/{name}", "revision": root["revision"],
                       "artifact": f"webboxvm-graphics/f02/{identifier}/{digest}.source", "suite_root_id": ROOT_ID,
                       "member_path": path})
    return result


def catalog() -> dict[str, object]:
    root = root_record()
    return {"schema": 1, "records": [root, *member_records(root)]}


def validate_catalog(value: object) -> None:
    try:
        roots.validate_catalog(value)
    except roots.RoleError as error:
        reject(str(error))
    if value != catalog():
        reject("GLES ledger catalog differs from the exact root closure")


def member_input(record: dict[str, object]):
    expected = {item["id"]: item for item in member_records(root_record())}.get(record.get("id"))
    if record != expected:
        reject("GLES member input differs from the exact root closure")
    return roots.SourceInput.from_manifest({"id": record["id"], "source_family": "f02532-gles-member",
        "immutable_url": record["immutable_url"], "revision": record["revision"], "sha256": record["sha256"],
        "bytes": record["bytes"], "license": record["license"], "local_cache": record["artifact"],
        "generated_code_role": "Immutable GLES full-root member; no generated code",
        "provenance": f"https://github.com/KhronosGroup/VK-GL-CTS/tree/{record['revision']}"})
