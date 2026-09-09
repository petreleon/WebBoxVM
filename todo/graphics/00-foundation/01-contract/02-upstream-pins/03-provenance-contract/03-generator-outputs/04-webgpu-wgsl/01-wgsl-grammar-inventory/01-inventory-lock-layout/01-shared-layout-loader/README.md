# F02.3.3.4.1.1.1 — Define the shared composite inventory layout

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../workflow.md)

Task: F02.3.3.4.1.1.1
Depends: F02.1, F02.2
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../../../../../01-input-inventory/README.md) and
[F02.2](../../../../../../02-fetch-verifier/README.md).

## Outcome

A stdlib-only, fail-closed layout model defined a versioned root index, bounded entry files, and a
checked-in `inventory.lock` before the active inventory changed. Its hermetic fixtures establish the
canonical identity and rejection rules without adding an upstream source or changing a runtime feature.

## Starting points

- [pre-cutover one-file inventory](../../../../../../01-input-inventory/manifest.toml)
- [F02.1 structural validator](../../../../../../01-input-inventory/validate_manifest.py)
- [F02.2 source model](../../../../../../02-fetch-verifier/01-fetch-contract/source_model.py)

## Checklist

- [x] Define the schema-v2 root, ordered bounded-entry declaration, and deterministic
  `inventory.lock` format with no payloads, absolute paths, timestamps, or output hashes.
- [x] Implement one stdlib-only layout loader that validates root, component names, raw hashes, and
  canonical lock identity before exposing an entry or inventory revision.
- [x] Preserve explicit schema-v1 compatibility until the cutover and add hermetic positive and
  negative fixtures for reordered, omitted, renamed, malformed, and byte-changed components.
- [x] Keep the checked F02.1 inventory unchanged and prove the focused suite has a nonzero passing
  count without starting a cache or generation action.

## Verification

- The same declared component bytes have one location-independent identity.
- Any component-layout error is rejected before a consumer can begin a cache or generation action.
