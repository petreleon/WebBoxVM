# F02.5.3.3.3 evidence

Revision: `95132673`
Validation: focused fresh-receipt tests, full local suite, source-file limits, fresh 99-object capture,
retained-cache receipt validation, offline replay, whitespace and roadmap checks
Result: PASS
Artifacts: [self-hashed full receipt](vulkan_full_suite_receipt.json), SHA-256
`83898210d2bbde6cdfd5e7ddd9ac15d4a895205fe63d05f9911445f05e201ce3`; retained cache
`/private/tmp/webboxvm-f02533.cfbvsd` (424,716 KiB, 101 regular files at capture close)
Profile: immutable Khronos `vk-default.txt` full-suite observation, broader than a Khronos
Vulkan-1.4-core-only selector; no CTS executable or guest application ran.

Task ID and date: F02.5.3.3.3, 2026-09-20.
Tested commit and dirty diff hash: `95132673`; code was committed and pushed before the successful
network capture. This receipt and evidence are later metadata-only work.
Upstream release proof: VCTS `vulkan-cts-1.4.6.2`, tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.
Guest image and build hashes: not applicable; no guest or CTS binary ran.
Browser, OS, adapter and driver: not applicable; this is a network/cache validation only.

## Fresh capture and replay

The successful command fetched exactly the immutable root plus the 98 ordered direct members: 99 raw
HTTPS requests and 434,672,695 transferred bytes. Each stream was SHA-256 and Git blob SHA-1 verified.
The cache was fresh (`reused=false`), then closed-world local replay made zero raw HTTPS requests. The
retained cache replay rehashed all 98 members (434,669,348 bytes) and reproduced receipt
`bc844fe3b3c32234b3ed3ec61da8267dbc3d67ab93e73bf4455fd8e320ccfdd4`.

Preflight required 939,666,249 bytes and observed 141,272,158,208 bytes free. All fourteen members
above 8 MiB remain in the committed metadata list; the largest is 61,932,251 bytes. The receipt itself
is `406130b30ca18b1843962456a5309ab24d41b222213b9a9d0e1cac06b6c98bb0`, with all WebBoxVM
claims false, `cts_executions: 0`, and `failure_or_skip: none`.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

```sh
make graphics-vulkan-full-suite-receipt-test
# 6 passed, 0 failed, 0 skipped
make test
# graphics tests passed; Rust 1,130 passed; Node 338 passed
cargo test -p emulator --test source_file_limits --quiet
# 6 passed
git diff --check
python3 scripts/check_graphics_roadmap.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/03-fresh-full-suite-receipt/vulkan_full_suite_receipt.py \
  --cache-root /private/tmp/webboxvm-f02533.cfbvsd --timeout 60
PYTHONDONTWRITEBYTECODE=1 python3 ../02-streaming-cache-replay/vulkan_cache_replay.py \
  --cache-root /private/tmp/webboxvm-f02533.cfbvsd
```

Negative/reference checks: the six focused cases reject out-of-order/repeated URLs, nonempty or
symlink cache roots, insufficient space before capture, reused/failed cache population, positive or
forged receipts, changed self-hash, source root, order, omitted oversized member, and `false`→`0`.
The receipt was independently rebuilt from the retained cache before this evidence was marked PASS.

First failing subcheck: `/private/tmp/webboxvm-f02533.BwXW8M` exposed a `urllib` timeout
positional-argument defect before payload write (zero files). `95132673` passes timeout by keyword,
adds a keyword-only regression fixture, and the new root completed successfully. The first root is not
reused.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the small receipt is
committed; raw VCTS payload is intentionally outside Git at the listed external root. Reproduce only
with a new private external root and the leaf command.
Software fallback detection and actual execution route: redirect-denying HTTPS streaming, descriptor
anchored local stage, V2 external cache, and closed-world offline replay; no renderer, guest, or
software graphics fallback executed.
Performance conditions and frozen protocol version: not applicable; this is not a graphics benchmark.
Decision and limits of the evidence: this proves receipt/cache provenance for an unfiltered Khronos
selector. It does not demonstrate Vulkan support, Vulkan 1.4 core coverage, conformance,
certification, qualification, performance, or near-native graphics behavior.
Commit/push verification: `95132673` was pushed to `origin/codex/graphics-f01-baseline`; no relevant
GitHub Actions workflow was triggered because the repository workflow filters unrelated paths.
Next ready task: F02.5.3.4 — Build local shards and publish a no-claim receipt.
