# F03.4.1 v1 historical evidence

Status: historical source-boundary record; it does not satisfy the active F02.5.4.2 role-aware task.

Revision: `3c889524b7da5638281290b08424b4b0dcddc3c0`
Validation: 5 hermetic hostile checks, 1 exact-payload structural check, local source-limit, full suite,
roadmap, and whitespace gates
Result: PASS
Artifacts: lock-bound empty scaffold plus reproducible 1,458-row technical inventory; no registry payload
is tracked in Git
Profile: planning-only raw registry structure for Vulkan 1.0–1.4; no guest, API support, CTS, browser, or
performance claim

Task ID and date: F03.4.1, 2026-09-11 Europe/Bucharest.

## Exact source and output

The source identity comes from F02's `vulkan-registry` record: selector `xml/vk.xml`, revision
`f84d432d5b8912362f96f581f29bbc4f3c8c7843`, SHA-256
`cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06`, 3,309,653 bytes, and
the recorded SPDX expression `Apache-2.0 OR MIT (SPDX file notice)`. The local read-only audit copy
matched that identity byte-for-byte; it remains ignored and is not a test dependency for the hermetic suite.

The explicit live command emitted `/private/tmp/webboxvm-f03-registry-v5.json`: 1,087,917 bytes,
SHA-256 `60c3605a649696d949f6d82ab2c6d33ab44c8953908f6398778cdb6c4ddebdf9`. Its row digest is
`5835f804af840c6df88ee907ea98d6bec0ec2dbb9821e4e0a58a3df6864bab15`; it is disposable and can be
recreated only from an exact payload. The artifact has 1,458 blocked rows: 20 version markers, 260 commands,
668 types, 393 enums, and 117 feature references. It records 1,350 `require` and 88 `deprecate` members.

## Commands and observed results

Working directory: `/Users/petreleon/code/WebBoxVM`.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/registry_inventory_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/registry_inventory_live_test.py .artifacts/vulkan-docs-official.K5cEx0/xml/vk.xml
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/registry_inventory.py --payload .artifacts/vulkan-docs-official.K5cEx0/xml/vk.xml --emit-json > /private/tmp/webboxvm-f03-registry-v5.json
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/01-registry-inventory/registry_inventory.py
cargo test -p emulator --test source_file_limits --quiet
make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

The hermetic suite passed 5/5. It covers ordered base/compute/graphics/public blocks, commands/types/enums,
`deprecate`, duplicate-name preservation, malformed/non-UTF-8/DTD input, oversized input, missing/reordered
blocks, unknown elements, partial/reordered/duplicate/promoted/assigned rows, JSON duplicate keys/type aliases,
stale identities, unsafe reader limits, symlinks, and the expected payload-free CLI state. The live test passed
1/1 and checked the exact digest, kind/lifecycle counts, and representative base and compute locators.

Without `--payload`, the CLI exits 3 and says `BLOCKED: exact external vk.xml payload is required`; this is an
expected no-payload result, not a failed graphics check. Every emitted effect remains false. The final local
source-file limit suite passed 6/6. `make test`, roadmap, and whitespace gates are recorded after this receipt
and parent update; no remote CI result is claimed.

## Decision and handoff

This inventory is deliberately separate from `matrix_contract.py`: it preserves raw structural provenance but
cannot satisfy the project's complete Docs role, choose a CTS selector, assign a feature owner/test plan,
advertise Venus/Vulkan support, or prove conformance or near-native performance. The internal blocks are not
a semantic public-core closure. `external-memory-extension-paths-excluded` excludes extension blocks and
platform-handle paths, not core-promoted structural references. The next Vulkan provenance/diagnostic work
remains F03.4.2 after its source-contract dependency; the generated-Docs authority blocker remains visible.

Commit/push verification: implementation `c69afb799f88fa19637ab2185b31df7b93a0c858` and reviewed boundary
fix `3c889524b7da5638281290b08424b4b0dcddc3c0` are local. No push is claimed here; a fresh explicit
remote-push authorization is required before publishing newer commits.
