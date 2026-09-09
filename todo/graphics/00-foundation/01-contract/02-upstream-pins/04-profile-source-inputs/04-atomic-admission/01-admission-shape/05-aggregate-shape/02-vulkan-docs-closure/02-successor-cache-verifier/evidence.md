# F02.4.4.1.5.2.2 receipt — successor closure cache staging

Revision: `7122a8d69d5f33226d8e7b6d9b9eecc0577bf2fd` implementation commit
Validation: 29 cache tests; predecessor suites; source-limit suite; full local `make test`; roadmap and whitespace checks
Result: PASS
Artifacts: descriptor-anchored isolated cache modules, byte-derived fixture, canonical marker, and hostile suites
Profile: synthetic successor fixture only; no Docs admission, guest API, browser, CTS, conformance, or performance claim

Task ID and date: F02.4.4.1.5.2.2, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `7122a8d69d5f33226d8e7b6d9b9eecc0577bf2fd`; clean implementation tree
before this receipt/status commit.
Upstream manifest revision: not applicable; the fixture is synthetic and validates frozen F02.2/V1 predecessor files.
Guest image and build hashes: not applicable; this cache-only task does not build or run a guest image.
Browser, OS, adapter and driver: not applicable; this cache-only task performs no browser or GPU execution.

## Implemented contract

- Both public entry points require fresh successor identity validation. The real WebBoxVM repository is pinned; the
  supplied root must be an absolute physical path outside it, with no symlinked component.
- Raw and generated members use the exact successor-only namespace. The generator receives bounded external producers,
  runs twice in separate scratch directories, and returns ordered byte-identical declared outputs with the bound
  recipe/configuration/toolchain/output-tree identities.
- Reuse requires an exact canonical self-hashed marker plus rehash of every member. Missing, stale, reordered,
  cross-closure, duplicate-key, malformed, oversize, symlinked, or partial state cannot become a hit.
- A read-only complete cache rehashes under a shared read-only lock. Staging uses an exclusive lock and atomic
  no-overwrite publication. The caller supplies a private external cache namespace; a non-cooperating same-UID writer
  may race or deny service after a verified snapshot, but cannot turn a changed observed member or marker into a hit.

## Focused verification

```text
Working directory: .../02-successor-cache-verifier
PYTHONDONTWRITEBYTECODE=1 python3 successor_cache_test.py                 7 passed
PYTHONDONTWRITEBYTECODE=1 python3 successor_cache_hostile_stage_test.py   8 passed
PYTHONDONTWRITEBYTECODE=1 python3 successor_cache_hostile_marker_test.py  6 passed
PYTHONDONTWRITEBYTECODE=1 python3 successor_cache_hostile_fs_test.py      8 passed

Identity predecessor: 8 + 3 + 3 passed
F02.2 source fetch: 15 passed; Vulkan audit/include: 10 + 6 passed
Vulkan boundary: 9 passed; post-cutover transition: 8 passed
cargo test -p emulator --test source_file_limits --quiet: 6 passed
make test: 1,130 Rust tests and 337 browser/JS tests passed; no failures or skips
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py: PASS before receipt
git diff --check: clean
```

Expected result and minimum nonzero case count: all 29 cache cases pass, including positive reuse and hostile
filesystem/marker/callback cases. Actual: 29 passed, 0 failed, 0 skipped. No raw payload, cache object, image, or
network fetch was retained in Git; the commands above reproduce the hermetic evidence.

Negative/reference checks: active `manifest.toml`, `inventory.lock`, input manifests, V1 rules, audit, and boundary
files had byte-identical before/after hashes; the receipt stays unadmitted and not cutover-ready. No software fallback,
execution route, or performance protocol applies.

First failing subcheck or blocker: none for this isolated cache child. Actual Vulkan Docs closure work remains outside
this PASS result and continues in the dependent child.
Decision and limits: this is not fetched Vulkan Docs, guest-visible Vulkan, browser rendering, CTS/conformance, or
near-native performance evidence.
Commit/push verification: `git ls-remote` returned
`7122a8d69d5f33226d8e7b6d9b9eecc0577bf2fd` for `origin/codex/graphics-f01-baseline`.
Next ready task: F02.4.4.1.5.2.3; its dependency is now met, but it remains blocked until two fresh pinned official
Vulkan Docs `makeSpec -clean -spec core -version 1.4 … html` builds produce an actual complete closure.
