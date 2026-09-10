# F02.4.4.1.5.5.3.2 — Live VCTS closure receipt

Revision: `ab87ff88` (`docs(graphics): capture pinned VCTS closure`).
Validation: 31 focused live-closure checks, 5 cache-contract checks, 10 roadmap-checker checks, and `make test` passed.
Result: PASS
Artifacts: pinned [tree plan](vcts_tree_plan.json), [ledger](vcts_closure_ledger.json), and [live receipt](vcts_live_capture_receipt.json).
Profile: SHA-pinned external VCTS suite-input availability only; no core selector, CTS, guest, browser, or performance claim.

Result scope: external suite-input availability only, captured 2026-09-10 Europe/Bucharest.

## Captured immutable input

- KhronosGroup/VK-GL-CTS `refs/tags/vulkan-cts-1.4.6.2`; annotated tag object
  `42c723aa10d2652590f02741827aef43b0421d23`; peeled commit
  `f6a29701220f34dd1407513bfe80d74ca7b392ce`.
- Root `external/vulkancts/mustpass/main/vk-default.txt`: 3,347 B; Git blob
  `365c6bf5fa66e22b8a62139e317da41765ab7ae0`; SHA-256
  `b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`.
- [Tree plan](vcts_tree_plan.json): 98 ordered direct members / 434,669,348 B;
  self-hash `77875b165876a3cf37450667c7e9c23e177bff8a7795e0d48b4214e202064804`.
- [Ledger](vcts_closure_ledger.json): self-hash
  `608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0`.
- [Live receipt](vcts_live_capture_receipt.json): self-hash
  `48c4d54d8738a8eab526347bc8080c245bb9cef269b66011e32116f78f481599`;
  cache-receipt hash `56fd45c16f1f1e1c68bfc76f3853215eaff08c71ec60e5ceb85c5ecb26ef8437`.

## Capture and replay

- One successful streamed HTTPS capture fetched the root plus 98 members: 99 raw requests,
  99 SHA-256 checks, 99 Git-blob-SHA-1 checks, and 434,672,695 B including the root.
- The retained external cache is `/private/tmp/webboxvm-vcts-v2-live-20260910-r2`; it is outside
  Git. The disposable capture stage was discarded after local replay.
- Offline verification of that cache after the final hardening returned 98 members / 434,669,348 B,
  cache receipt `56fd45c16f1f1e1c68bfc76f3853215eaff08c71ec60e5ceb85c5ecb26ef8437`, and zero raw requests.
- The first equivalent external cache, `/private/tmp/webboxvm-vcts-v2-live-20260910`, had the same
  receipt file SHA-256 `41abf2e502fdea0ca6961359b1c10259a686b9056a0d84e13b4165bc246bcf8b` and was deleted after
  comparison. It is not recoverable locally; the retained cache remains available.
- No network or cache failure occurred. An earlier outer execution-window interruption created no
  ledger or receipt and is not used as a fetch/cache failure result or as evidence.

## Verification

- Hermetic checks passed: tree plan 3; GitHub-tree capture 4; live stage 5; stage failure cleanup 2;
  coordinator 6; transaction publisher 7; capture receipt 4; cache contract 5; roadmap checker 10.
- `make test` passed after hardening: Rust 1,127 passed / 0 failed / 3 ignored; source-file limits
  6/6; Node 337 passed / 0 failed.
- The successful network capture preceded final failure-path/output hardening. The final code covers
  those paths hermetically and revalidated this recorded cache and receipt offline.

## Scope boundary

- This is a SHA-pinned, Khronos-hosted VCTS suite-input capture. The tag signature was not
  cryptographically verified.
- The receipt explicitly sets `admitted: false`, `cutover_ready: false`, and
  `satisfies_vulkan_14_core_manifest: false`.
- It does not prove a Vulkan-1.4-core selector, CTS execution or conformance, guest/browser
  compatibility, API support, or performance. Payload bytes remain outside Git.
