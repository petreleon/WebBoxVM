# F02.5.3 evidence

Revision: `3824a177`
Validation: canonical-root, GL/GLES ledger, Vulkan taxonomy/cache/fresh-capture, and local-shard
receipts; full local suite, source-file limits, whitespace and roadmap checks
Result: PASS
Artifacts: [canonical roots](01-canonical-full-suite-roots/evidence.md),
[GL/GLES ledger receipt](02-gl-gles-full-ledgers/03-gl-gles-no-claim-receipt/evidence.md),
[Vulkan full ledger receipt](03-vulkan-default-full-ledger/evidence.md), and
[bounded local-shard receipt](04-local-shards-and-no-claim-receipt/vulkan_local_shard_receipt.json)
Profile: unfiltered Khronos CTS/must-pass roots plus explicitly non-qualifying WebBoxVM transforms.

Task ID and date: F02.5.3, 2026-09-20.
Tested commit: `3824a177`; the final metadata is later documentation work.
Upstream release proof: VCTS `vulkan-cts-1.4.6.2`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.
Guest image and build hashes: not applicable; no guest ran.
Browser, OS, adapter and driver: not applicable; no renderer ran.

The parent establishes the three exact full-suite roots and preserves the released Vulkan
`vk-default.txt` root unchanged. Its Vulkan closure has 98 direct members and 434,669,348 aggregate
bytes; fresh capture requested all 99 raw objects (root plus members), retained all fourteen
oversized upstream members, and made no CTS claim. F02.5.3.4 then selects exactly one proven ledger
member as an external input and creates five separately labeled WebBoxVM byte-preserving transforms.

The shard receipt records all five local artifacts at or below 8 MiB, their offsets and SHA-256
identities, exact reassembly to `api.txt`, zero network during offline replay, zero CTS executions,
six false claims, and false admission/cutover/core-manifest states. The local artifacts remain neither
a Khronos selector nor a replacement for the full VCTS root.

Exact commands, working directory, tool versions, hostile cases, external raw-payload locations, and
reproduction steps are recorded in the child receipts. The final leaf ran all graphics checks,
`make test`, `cargo test -p emulator --test source_file_limits --quiet`, `git diff --check`, and the
roadmap checker. The local source route is an external-cache replay and stream-copy only; no renderer
or fallback route ran, and performance is not measured.

Decision and limits of the evidence: this completes full-suite provenance and bounded local transport
evidence, not an API implementation, compatibility/conformance result, certification, profile support,
performance result, or near-native VM graphics claim.
Commit/push verification: `3824a177` is pushed to `origin/codex/graphics-f01-baseline`; no relevant
GitHub Actions workflow was triggered by these path-filtered changes.
