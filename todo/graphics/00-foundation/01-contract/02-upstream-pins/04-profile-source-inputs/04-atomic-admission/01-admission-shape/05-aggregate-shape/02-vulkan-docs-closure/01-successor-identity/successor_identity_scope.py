"""Typed scope validation for the isolated successor fixture."""

from __future__ import annotations

from successor_identity_model import SCOPE_FIELDS, GeneratedMember, Scope, reject
from successor_identity_parse import canonical, digest, file_selector


def scope(value: object, members, generations) -> Scope:
    if not isinstance(value, dict) or set(value) != SCOPE_FIELDS:
        reject("fixture scope has an invalid schema")
    if value.get("scope_kind") != "fixture-unbound-core-scope":
        reject("fixture scope claims actual Docs evidence")
    if value.get("evidence_mode") != "fixture-unbound":
        reject("fixture scope has an invalid evidence mode")
    member_ids = tuple(item.identifier for item in members)
    generated_ids = tuple(
        item.identifier for item in members if isinstance(item, GeneratedMember)
    )
    if value.get("ordered_member_ids") != list(member_ids):
        reject("fixture scope does not retain ordered member identity")
    if value.get("generated_member_ids") != list(generated_ids):
        reject("fixture scope does not retain generated member identity")
    excluded = value.get("excluded_members")
    keys = {"wsi", "video", "extensions"}
    if not isinstance(excluded, dict) or set(excluded) != keys:
        reject("fixture scope has invalid typed exclusions")
    if any(not isinstance(rows, list) for rows in excluded.values()):
        reject("fixture scope exclusions are not lists")
    for name, rows in excluded.items():
        if any(not isinstance(item, str) or not item for item in rows) or len(set(rows)) != len(rows):
            reject("fixture scope exclusions are not unique strings")
        for item in rows:
            file_selector(item, f"{name} exclusion")
    configs = {item.configuration_digest for item in generations}
    if len(configs) != 1 or digest(value.get("configuration_sha256"), "scope configuration") not in configs:
        reject("fixture scope has a mixed generation configuration")
    expected = canonical({key: item for key, item in value.items() if key != "scope_sha256"})
    scope_digest = digest(value.get("scope_sha256"), "scope sha256")
    if scope_digest != expected:
        reject("fixture scope has a stale digest")
    return Scope(member_ids, generated_ids, next(iter(configs)), scope_digest)
