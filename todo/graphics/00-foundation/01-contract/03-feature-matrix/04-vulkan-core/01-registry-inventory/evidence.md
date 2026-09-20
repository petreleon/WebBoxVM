# F03.4.1 v2 receipt — sealed auxiliary Vulkan registry inventory

Revision: `b2c381de49e0b87be3e8d4796c23c8d8f9891520`
Validation: 11 hermetic hostile checks, one fresh-cache structural check, full local suite, source limits,
roadmap, and whitespace checks
Result: PASS
Artifacts: a small sealed scaffold and a reproducible external 1,458-row technical inventory; no upstream
payload or full row capture is tracked in Git
Profile: planning-only raw registry structure; no guest, API support, CTS execution, browser, certification,
or performance claim

Task ID and date: F03.4.1, 2026-09-21 Europe/Bucharest.

## Sealed source and external capture

The v2 reader loads F02.5.4.2 through the fixed-path role-aware binding. It fixes source-contract SHA-256
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3` and inventory-lock SHA-256
`44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`. It accepts only the auxiliary
`vulkan-registry` record: revision `f84d432d5b8912362f96f581f29bbc4f3c8c7843`, SHA-256
`cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06`, 3,309,653 bytes, SPDX expression
`Apache-2.0 OR MIT (vk.xml SPDX license choice)`, and marker `VK_VERSION_1_4`.

The record's scope is `registry-metadata`, it has `discharges_required_role: false`, all claims false, and
zero CTS executions. It is explicitly absent from the six mandatory profile-role bindings, so it cannot
become either the Vulkan normative Docs root or the full-suite root.

Fresh external cache: `/private/tmp/webboxvm-f0341.cqT6ZX`. The F02 refresher atomically fetched and verified
exactly nine regular files: seven sealed selectors plus the two release-license proofs. The exact registry
path is `webboxvm-graphics/f02/vulkan-registry/cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06.source`.
Retain this cache for local audit; the generated full inventory is
`/private/tmp/webboxvm-f0341-registry-v2.json`, 1,089,060 bytes, SHA-256
`b1e9f31cff4a2f30e69cc9a64ef65e0b97cc5fba84310c5b8e22d67b74c4da2c`.

## Observed output and boundaries

The fresh-cache run emitted 1,458 blocked rows with digest
`5835f804af840c6df88ee907ea98d6bec0ec2dbb9821e4e0a58a3df6864bab15`: 20 version markers, 260 commands,
668 types, 393 enums, 117 features, 1,350 `require` members, and 88 `deprecate` members. Every row retains
null implementation-owner and independent-test-plan fields. Every emitted effect is false, including
normative Docs role, full-suite root, CTS selection, support, conformance, certification, and near-native
performance.

Extension blocks, WSI, external-memory extension/platform-handle paths, and the separate SPIR-V grammar
remain excluded. Core-promoted structural rows remain raw facts; they are not a semantic public-core closure.
The active v2 code never imports the old F02.1 manifest/boundary path. The retained [v1 receipt](evidence-v1.md)
is historical evidence only.

## Commands and results

Working directory: `/Users/petreleon/code/WebBoxVM`.

```sh
make graphics-vulkan-registry-inventory-test
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/role_aware_source_cache.py --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX --timeout 60
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/v2/registry_inventory_live_test.py --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/v2/registry_inventory.py --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX --emit-json > /private/tmp/webboxvm-f0341-registry-v2.json
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

The focused checks passed 11/11. They reject stale contract headers, identity/license/size/marker changes,
auxiliary-to-root promotion, missing auxiliary evidence, mixed cache contracts, relative/repository/symlink/
partial/extra cache paths, payload symlinks, arbitrary CLI payload input, DTD/entity/non-UTF-8/malformed XML,
missing or reordered core blocks, unknown XML members, partial/reordered/duplicate/promoted/assigned rows,
and promoted effects. The live check passed with the counts and digest above. The no-cache CLI deliberately
exits 3 as `BLOCKED`; it does not fall back to an arbitrary `vk.xml`.

`make test`, source-file limits, roadmap, and whitespace gates passed locally. No GitHub Actions result is
claimed; remote validation remains separate from these local results.

## Handoff

F03.4.1 is complete as a raw auxiliary technical inventory. F03.4.2 remains open to attach normative Docs
provenance and CTS diagnostics; it must not reinterpret this registry as either required root. The broader
F03 profile matrix remains incomplete, and the goal's real guest, browser, conformance, and performance gates
remain unresolved.
