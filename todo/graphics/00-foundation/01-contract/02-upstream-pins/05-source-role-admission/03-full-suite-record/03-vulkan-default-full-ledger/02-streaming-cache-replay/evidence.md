# F02.5.3.3.2 evidence

Revision: `5bd987af`
Result: PASS
Validation: bounded streaming fixture, hostile cache cases, exact retained-cache replay,
full repository test suite, source-file limit test, whitespace check, and roadmap checker.
Artifacts: no raw VCTS member is committed; the verified payload remains in an external cache.
Profile: unfiltered `vk-default.txt` closure observation only, not a CTS execution or result.

## Recorded result

Task ID and date: F02.5.3.3.2, 2026-09-11.

The replay first rebuilds the fixed F02.5.3.3.1 observation, then requires the V2 cache
identity, ledger digest, count, and aggregate bytes to match it. One shared cache lock
holds both full member verification and the type-strict marker comparison. Thus an external
cache cannot choose a different selector, resealed ledger, or marker representation.

The exercised external cache was `/private/tmp/webboxvm-vcts-v2-live-20260910-r2`.
The offline command completed with 98 members, 434,669,348 bytes, ledger SHA-256
`608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0`, and local
receipt SHA-256 `bc844fe3b3c32234b3ed3ec61da8267dbc3d67ab93e73bf4455fd8e320ccfdd4`.
It used 65,536-byte reads, retained the V2 64 MiB per-member policy, and filtered no raw
member; fourteen released members remain above the separate 8 MiB local-transform cap.

## Hostile checks

`make graphics-vulkan-cache-replay-test` passed 5 tests. They cover a member above 8 MiB
with bounded reads; one closure lock around the F02 verifier; same-length content corruption;
an interrupted publication with no marker plus a member symlink; a self-hashed marker where
JSON `false` is changed to `0`; no-claim receipt fields; and a V2 ledger mismatch.

The marker check intentionally compares JSON types recursively after V2 parsing, because
ordinary Python equality treats `False` and `0` as equal. Cache member SHA-256 and Git blob
SHA-1 are both rehashed by the cache contract under the same shared closure lock.

## Reproduction and boundaries

```sh
make graphics-vulkan-cache-replay-test
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/03-full-suite-record/03-vulkan-default-full-ledger/02-streaming-cache-replay/vulkan_cache_replay.py \
  --cache-root /private/tmp/webboxvm-vcts-v2-live-20260910-r2
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
python3 scripts/check_graphics_roadmap.py
```

All commands passed. The final full suite included 1,130 Rust and 338 Node tests; the
roadmap checker reported 386 documents, 229 tasks, 51 PASS-complete, and 81 superseded.
The receipt leaves every WebBoxVM claim false, records `cts_executions: 0`, and retains only
the Khronos selector claim on the immutable source root. It does not establish Vulkan 1.4
core support, conformance, certification, profile support, or performance. `5bd987af` is a
local commit only; no remote push occurred.
