# F02.5.3.3 evidence

Revision: `95132673`
Validation: child receipts, fresh immutable VCTS capture, retained-cache replay, full local test suite,
source-file limits, whitespace and roadmap checks
Result: PASS
Artifacts: [taxonomy receipt](01-vulkan-ledger-taxonomy/evidence.md),
[cache-replay receipt](02-streaming-cache-replay/evidence.md), and
[fresh full-suite receipt](03-fresh-full-suite-receipt/vulkan_full_suite_receipt.json)
Profile: unfiltered Khronos `vk-default.txt` observation; no CTS execution or qualification result.

Task ID and date: F02.5.3.3, 2026-09-20.
Tested commit and dirty diff hash: `95132673`; this aggregate is later metadata-only work.
Upstream release proof: VCTS `vulkan-cts-1.4.6.2`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.
Guest image and build hashes: not applicable; no guest ran.
Browser, OS, adapter and driver: not applicable; no renderer ran.

The three closed children bind the exact 98-member ordered ledger/taxonomy, stream and replay it
through a safe external cache, then make a fresh 99-request capture. The final receipt records
434,672,695 raw bytes, a 434,669,348-byte member aggregate, fourteen retained oversized members,
cache reuse `false`, offline raw requests `0`, all WebBoxVM claims false, and `cts_executions: 0`.
Its self-hash is `406130b30ca18b1843962456a5309ab24d41b222213b9a9d0e1cac06b6c98bb0`.

Exact command(s), working directory and tool versions: see the three child receipts; the final leaf
ran `make graphics-vulkan-full-suite-receipt-test`, `make test`,
`cargo test -p emulator --test source_file_limits --quiet`, `git diff --check`, and the roadmap checker.
Expected result and minimum nonzero case count: all 98 members, 99 raw streams, 14 oversized members,
and six hostile fresh-receipt tests.
Actual passed/failed/skipped counts and exit codes: six focused tests, 1,130 Rust tests and 338 Node
tests passed; successful fresh capture and offline replay each exited 0.
Negative/reference checks and observed output: child tests reject altered roots, ledger rows, cache
content, paths, marker types, replay receipt fields, nonfresh roots, missing/repeated raw URLs, and
positive claims.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the retained external
cache is recorded by the child receipt; raw VCTS members are not committed. Recreate only with a new
private external root and the leaf command.
Software fallback detection and actual execution route: HTTPS streaming and local external-cache
replay only; no guest renderer or fallback was involved.
Performance conditions and frozen protocol version: not applicable.
First failing subcheck or blocker: the initial fresh attempt exposed a timeout forwarding defect before
payload write; it was fixed and regression-tested in `95132673` before successful capture.
Decision and limits of the evidence: this completes a provenance ledger, not a Vulkan core-only
selector, API implementation, CTS run, conformance result, certification, or performance claim.
Commit/push verification: `95132673` is pushed; the path-filtered GitHub Actions workflow is unrelated
and did not run for this leaf.
Next ready task: F02.5.3.4 — Build local shards and publish a no-claim receipt.
