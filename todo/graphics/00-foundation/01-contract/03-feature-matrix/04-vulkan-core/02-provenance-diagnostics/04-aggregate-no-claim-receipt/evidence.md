# F03.4.2.4 evidence — no-claim provenance aggregate

Revision: `88ab1470fa1335f48d4983960b2b3e5807833c20`
Validation: 8 focused hostile checks, fresh external-cache aggregate replay, `make test`, source-file limits, whitespace, and roadmap check
Result: PASS
Artifacts: `vulkan_no_claim_provenance_receipt.json` SHA-256 `16a01c55c05d5baad6be2a99eb5438032388248fa97824e9fb18ca79288271d5`; embedded receipt self-hash `e13083e5283ca941730c943cd7a23375abf6678fcadf7057b20d9411d89c03f0`
Profile: Vulkan 1.4 core planning provenance only; `matrix-incomplete`

Task ID and date: F03.4.2.4, 2026-09-21 Europe/Bucharest.

## Aggregate

The small receipt joins the active role-aware source-contract and inventory-lock identities to the
external 1,458-row auxiliary `vk.xml` observation: source order `1..1458`, raw-row digest
`5835f804af840c6df88ee907ea98d6bec0ec2dbb9821e4e0a58a3df6864bab15`, and a null-owner,
null-test-plan, `blocked` row policy. Its source role stays `registry-metadata`; the distinct
inventory scope is raw Vulkan 1.0–1.4 structural blocks.

It also pins the F03.4.2.1 source-channel hash
`dd07b39283a3e41edf217e20dbb1eec90e8e6e73b6aae748d46bb57588983683`, the F03.4.2.2
citation-map hash `2d6b3270cd4af8d14d752d593cd226c54d953aa034ce3f23cb221eecc0b22e44`, and the
F03.4.2.3 diagnostic hash `66d771723737b03f4a301db51b927ee51da89db9eaf5f2a10f8deeaf8e7f0b68`.
Docs remains citation-only with two candidates and no admitted include closure. VCTS remains a
98-member root-wide diagnostic, not a selected or executed core suite.

## Commands and results

Working directory: `/Users/petreleon/code/WebBoxVM`.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/04-aggregate-no-claim-receipt/vulkan_no_claim_provenance_receipt_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/04-vulkan-core/02-provenance-diagnostics/04-aggregate-no-claim-receipt/vulkan_no_claim_provenance_receipt.py --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX --docs-cache-root /private/tmp/webboxvm-f03422-final.FYptCW
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
make test
```

The focused suite has eight test methods. They cover claim-free construction; stale and promoted receipts;
mixed prerequisite pins; Docs/VCTS scope promotion; missing, duplicate, and reordered raw rows; exact
effects/schema/registry identity, including JSON type aliases; cache/subprocess failure; and absent-cache
rejection.
The live replay revalidates both external caches and reruns the raw registry inventory without retaining
its 1.09 MB row capture in Git.

Actual leaf results: 8 passed, 0 failed; fresh replay exited 0; `make test` exited 0; the source-file
limit suite passed 6/6; `git diff --check` was clean; and the roadmap check reported valid links,
dependencies, and limits. This receipt does not make F03.4 or the graphics goal complete.

## Limits and handoff

Every receipt claim is false; matrix rows and CTS executions are zero. This is not guest or browser
execution, API support, conformance, certification, or a performance result. The blocked obligations are
the admitted Docs closure, semantic-core matrix, owners/test plans, core-only CTS selection/execution,
guest/browser validation, conformance/certification, and same-GPU measurement.

First failing leaf subcheck: none. Commit/push verification: code commit `88ab1470` is confirmed on
`origin/codex/graphics-f01-baseline`; GitHub Actions reported no run for that SHA. F03.4 and the overall
graphics goal remain open.
