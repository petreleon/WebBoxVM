# F02.5.3.4 evidence

Revision: `3824a177`
Validation: eight focused shard/receipt tests, shared source-role tests, full local suite,
source-file limits, static receipt validation, whitespace and roadmap checks
Result: PASS
Artifacts: [self-hashed local-shard receipt](vulkan_local_shard_receipt.json), SHA-256
`0569b28d1b0c7d9e5ae85a047019fb15b187d836071f63f0fce462849ce3688d`; external artifact root
`/private/tmp/webboxvm-f02534.NpA5Lz` (78,716 KiB, eight regular files)
Profile: immutable Khronos `vk-default.txt` provenance, plus a WebBoxVM local transform; no CTS run.

Task ID and date: F02.5.3.4, 2026-09-20.
Tested code commit: `3824a177`, pushed before this metadata. The retained external artifacts were
independently revalidated against that exact builder and static receipt after the commit.
Upstream release proof: VCTS `vulkan-cts-1.4.6.2`, tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.
Guest image and build hashes: not applicable; no guest or CTS binary ran.
Browser, OS, adapter and driver: not applicable; no renderer ran.

## Provenance and bounded output

The input is the exact direct `api.txt` VCTS ledger row: Git blob SHA-1
`9440aee9b3291b2927119c97ab44e2858a63a39b`, SHA-256
`a329e8606983b982a34e45f254ac871b3d46b9fce137b78efa15878a5d238a0c`, and
40,296,059 bytes. Before staging it, the implementation replays the complete 98-member V2 cache
offline and binds the row to the unfiltered `vulkan-cts-default` root and its exact ledger.

The new separately pinned `byte-preserving-shard` builder made five WebBoxVM artifacts in source
order: four are 8,388,608 bytes at offsets 0, 8,388,608, 16,777,216, and 25,165,824; the final one
is 6,741,627 bytes at offset 33,554,432. Each has a pinned SHA-256, and streamed reassembly equals
the upstream member SHA-256 exactly. The raw cached source is only an external verification input;
the 8 MiB bound applies to the five local transform artifacts, never to the upstream VCTS member.

The receipt reports zero raw HTTPS requests, zero CTS executions, all six claims false, and all
admission/cutover/core-manifest states false. It references the fresh full-suite receipt
`406130b30ca18b1843962456a5309ab24d41b222213b9a9d0e1cac06b6c98bb0` and its offline cache replay
`bc844fe3b3c32234b3ed3ec61da8267dbc3d67ab93e73bf4455fd8e320ccfdd4`.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

```sh
make graphics-vulkan-local-shard-receipt-test
# 8 passed, 0 failed, 0 skipped
make graphics-source-role-test
# 13 passed, 0 failed, 0 skipped
make test
# graphics tests passed; Rust 1,130 passed; Node 338 passed
cargo test -p emulator --test source_file_limits --quiet
# 6 passed
git diff --check
python3 scripts/check_graphics_roadmap.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/04-local-shards-and-no-claim-receipt/vulkan_local_shard_receipt.py \
  --cache-root /private/tmp/webboxvm-f02533.cfbvsd \
  --artifact-root /private/tmp/webboxvm-f02534.NpA5Lz
```

Negative/reference checks: malformed shard mode, changed input, oversize intervals, existing and
symlinked builder paths, omitted/reordered/oversized/root-mismatched catalog entries, altered staged
source/shard/builder bytes, a failed cache bridge, positive claims, `false`→`0`, CTS count, changed
source identity, reordered receipt shards, builder digest, and self-hash all fail.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: only the small receipt
is committed. The raw `api.txt`, copied verification source, builder copy, and five shards remain in
the listed private external artifact root; recreate from the retained verified cache or a new
equivalent private cache root.
Software fallback detection and actual execution route: offline VCTS cache replay, stream copy, and
local byte-preserving output; no guest renderer or software graphics fallback executed.
Performance conditions and frozen protocol version: not applicable; this is a provenance/transport
artifact, not a graphics benchmark.
Decision and limits of the evidence: this proves five bounded WebBoxVM transforms reassemble one
released VCTS member. It does not prove Vulkan API support, Vulkan 1.4 core coverage, CTS execution,
conformance, certification, qualification, performance, or near-native graphics behavior.
Commit/push verification: `3824a177` is pushed to `origin/codex/graphics-f01-baseline`; no relevant
GitHub Actions workflow was triggered because its path filter is unrelated to this task.
