# F02.1 — Define the immutable graphics-input inventory

[Parent task](../README.md) · [Worker instructions](../../../../workflow.md)

Task: F02.1
Depends: F01
Evidence: [receipt](evidence.md)

Prerequisite lists: [F01](../../01-baseline/README.md).

## Outcome

One small, reviewable inventory index and bounded component name every authoritative input and
immutable byte identity.

## Starting points

- [primary sources](../../../../sources.md)
- [VirGL2 boundary](../../../../../../research/virgl2-capset.md)
- [Venus foundations](../../../../../../research/venus-foundations.md)

## Checklist

- [x] Define a maintained manifest schema with immutable URL, revision, SHA-256, source family,
  license, local cache location, and generated-code role for every entry.
- [x] Add entries for Linux UAPI; Mesa/VirGL/Venus; virglrenderer/venus-protocol; GL/GLES,
  GLSL/ESSL, Vulkan, SPIR-V, WebGPU/WGSL; and selected independent reference suites.
- [x] Reject mutable branch-only references and keep fetched bytes outside maintained source.
- [x] Validate manifest structure and the complete required source-family inventory with a nonzero,
  implementation-independent test.

## Verification

- The manifest parser accepts the reviewed inventory and rejects a missing family, bad digest, or
  mutable source reference.
- F02.2 fetches its immutable bytes; this task does not treat planned downloads as evidence.

## Inventory contract

[manifest.toml](manifest.toml) is the schema-v2 metadata index. Its ordered component list names
the bounded `inputs/part-0001.toml` record file, and [inventory.lock](inventory.lock) canonically
binds the raw bytes of both files. Every `inputs` entry has exactly the ten required fields listed in
the first checklist item. `local_cache` is relative to the external `$XDG_CACHE_HOME`; no fetched
upstream bytes belong in the repository. The inventory revision is the raw SHA-256 of the checked
lock, printed by the validator rather than self-recorded.

Run the hermetic structural check with:

```sh
python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
```

The validator does no I/O beyond reading the manifest. F02.2 owns network fetches, actual hash
comparison, corruption handling and cache writes.
