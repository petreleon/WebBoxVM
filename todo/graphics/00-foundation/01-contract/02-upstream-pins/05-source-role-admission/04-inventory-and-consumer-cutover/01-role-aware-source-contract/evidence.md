# F02.5.4.1 evidence

Revision: `071cf3f4`
Validation: eight focused contract/cache tests, a fresh nine-file selector cache, full local suite,
source-file limits, diff, and roadmap checks
Result: PASS
Artifacts: [sealed source contract](source_contract.json), raw SHA-256
`c35f915fe5a6a3dfe896b97de77f89903a19211e5993b50bc8cdabbaaff60831`; external selector cache
`/private/tmp/webboxvm-f02541.kY8W4H` (nine regular files)
Profile: source provenance only; no guest, browser, CTS execution, or qualification result

Task ID and date: F02.5.4.1, 2026-09-20.
Tested code commit: `071cf3f4`, pushed before this metadata. The code worktree was clean before this
receipt update. The seal self-hash is `480139cc57a5ab5ef9fcd58695d66e55053d10db12cb65531100eb62ccacf7a6`;
it binds source-contract hash `d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3`
and eight-record inventory lock `44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`.
Upstream roots: the three reviewed normative sources and the unfiltered Khronos OpenGL, GLES, and
Vulkan default full-suite roots from F02.5.2/F02.5.3. The Vulkan root is explicitly broader than a
core-only selector; no local map or shard replaces it.
Guest image and build hashes: not applicable; no guest or CTS binary ran.
Browser, OS, adapter and driver: not applicable; no renderer ran.

## Contract and fresh-cache result

The ordered sealed catalog has eight records. Its only six mandatory bindings are the profile pairs
OpenGL 4.6 (`opengl-46-core-spec`, `opengl-cts-gl46-main`), GLES 3.2 (`gles-32-spec`,
`gles-cts-main`), and Vulkan 1.4 (`vulkan-14-spec`, `vulkan-cts-default`). `vulkan-registry` and
`vulkan-14-core-definition` are auxiliary. The GL/GLES engineering-map receipt, the Vulkan
98-member/434,669,348-byte closure ledger and replay/fresh receipts, and the five shard receipt are
also auxiliary no-claim evidence; none can discharge a mandatory role.

The combined fresh selector cache downloaded exactly seven pinned root sources and two release-license
proofs into a private external root. Its largest selector is the 3,309,653-byte registry, so this
bounded transport does not carry `api.txt` or any VCTS closure member. The separate full VCTS cache
and receipt remain the authoritative closure evidence, including its 14 released members over 8 MiB.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

```sh
make graphics-role-aware-source-contract-test
# 8 passed, 0 failed, 0 skipped
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/01-role-aware-source-contract/role_aware_source_cache.py \
  --selector-cache-root /private/tmp/webboxvm-f02541.kY8W4H --timeout 60
# seven selectors plus two licenses, then an atomic nine-file publish
make test
# graphics checks passed; Rust suite passed; Node checks passed
cargo test -p emulator --test source_file_limits --quiet
# 6 passed
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Negative/reference checks: raw-lock reformatting, stale self-hashes, legacy manifest aliases,
missing/reordered roots or closures, a registry or transform substituted for a mandatory role,
positive claims, CTS counts, promoted Vulkan receipt states, a nonempty/repository/symlink cache,
zero timeout, and reuse/partial atomic publication all fail. Tests also prove the composite refresh
requests the fixed 4 + 3 + 2 sequence before publishing a cache target.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the small seal and raw
lock are committed. Re-run the cache command with a newly empty, external private root to recreate
the nine verified selector files. No image or sample exists.
Software fallback detection and actual execution route: immutable HTTPS source retrieval, hashes,
notice checks, release-license checks, and no-claim receipt validation; no renderer or fallback ran.
Performance conditions and frozen protocol version: not applicable; this is a provenance contract,
not a graphics benchmark.
First failing subcheck or blocker: the first draft's pretty JSON exceeded the 180-line maintained-file
limit. It was replaced by a 160-line raw-byte-locked seal whose source-contract hash binds the full
reviewed catalog; the generated full record stays in bounded source modules.
Decision and limits of the evidence: this admits immutable source provenance only. F03 remains
`inventory-sources-incomplete` until F02.5.4.2 wires this contract into its gate, and all profiles
remain blocked. This does not prove API support, CTS execution, conformance, certification, profile
support, browser operation, performance, or a near-native VM claim.
Commit/push verification: `071cf3f4` is pushed to `origin/codex/graphics-f01-baseline`. `gh run list`
returned no run for that SHA, so no GitHub Actions validation is claimed.
Next ready task: F02.5.4.2.
