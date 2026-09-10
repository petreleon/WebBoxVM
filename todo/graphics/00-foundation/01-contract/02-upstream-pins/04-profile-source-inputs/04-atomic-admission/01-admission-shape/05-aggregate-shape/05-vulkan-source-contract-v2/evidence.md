# F02.4.4.1.5.5 — V2 aggregate receipt

Revision: `dfb0a236c3e170fdb668db7b28634c0069757417`.
Validation: all five V2 children have receipts; V2 handoff 8/8; checker 10/10; `make test` passed.
Result: PASS
Artifacts: root `30b272f8…c7bd5218`; ledger `608d5204…e917f0`; handoff `8b286471…e682ee`.
Profile: immutable external VCTS source-availability evidence only; no compatibility or conformance result.

The V2 aggregate binds the Khronos-hosted, SHA-pinned `vk-default.txt` root to its verified external
closure without silently narrowing it. The final [handoff receipt](05-aggregate-handoff/vcts_aggregate_handoff.json)
binds root `30b272f8…c7bd5218`, tree plan `77875b16…2064804`, ledger `608d5204…e917f0`
(98 members / 434,669,348 B), cache receipt `56fd45c1…b8437`, live receipt `48c4d54d…481599`,
and taxonomy `752a3ae5…6cb2f7` as self-hash `8b286471…e682ee`.

The adapter rejects V1/V2 mixing, stale or substituted roots, incomplete/reordered closure records,
detached/reclassified taxonomy, cache/live-receipt tampering, and any admission or Docs-boundary claim.
It preserves `admitted: false`, `cutover_ready: false`, and
`satisfies_vulkan_14_core_manifest: false`. Generated Vulkan Docs remain external provenance, not admitted
implementation source. The recorded cache receipt is revalidated offline; this does not claim a fresh fetch,
current payload availability, CTS execution, Khronos certification, or GPU support/performance.

From the repository root, reproduce the final layer with
`python3 -B todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/05-aggregate-handoff/vcts_aggregate_handoff_test.py`,
then the adjacent `vcts_aggregate_handoff.py`, `scripts/test_check_graphics_roadmap.py`,
`scripts/check_graphics_roadmap.py`, and `make test`. The external cache replay also passed for the retained
`/private/tmp/webboxvm-vcts-v2-live-20260910-r2` cache. This makes F02.4.4.1.5.4 ready; it does not complete it.
