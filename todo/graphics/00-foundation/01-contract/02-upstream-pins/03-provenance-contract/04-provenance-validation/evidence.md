# F02.3.4 evidence

Revision: `b50dcbedd79e719ace50628daa773b8eac3cb81a` implementation commit
Validation: fresh 17-input fetch, offline 17-input rehash, closure 17/12, unavailable-cache failure,
focused suites, source limits, local `make test`, roadmap, and whitespace
Result: PASS
Artifacts: disposable external cache `/private/tmp/webboxvm-f0234-live.OwEDjs`; no upstream payload is stored in Git
Profile: provenance/cache closure only; no guest graphics API, renderer, browser, compatibility, or performance claim

Task ID and date: F02.3.4, 2026-09-09 Europe/Bucharest.

`validate_provenance_closure.py` first accepts only the complete F02.2 cache: every one of the 17
canonical cache paths must be a regular file below the external root, with no symlinked path component,
the declared byte count, and its pinned SHA-256. It then accepts exactly six ABI and six generator
sidecars, re-validating the immutable inventory lock, IDs, digests, licenses, reviewed command/output
fingerprints, recorded artifact paths, and current local output hashes. An unlisted JSON sidecar is a
failure, rather than an implicit new provenance record.

## Fresh cache and closure result

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch.py --cache-root /private/tmp/webboxvm-f0234-live.OwEDjs
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch.py --cache-root /private/tmp/webboxvm-f0234-live.OwEDjs
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/04-provenance-validation/validate_provenance_closure.py --cache-root /private/tmp/webboxvm-f0234-live.OwEDjs
```

The inventory command printed `PASS: 17 immutable inputs` with lock SHA-256
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`. The first cache command
printed 17 `fetched` results. The second printed the same 17 IDs as `reused` and ran without network
elevation, re-reading and re-hashing their bytes. The closure command printed:

```text
PASS: 17 cached sources and 12 provenance records
```

The external audit found 17 regular `.source` files totaling 18,953,659 bytes. The cache remains
outside the repository and is not an input to Git.

## Failure and regression evidence

The same CLI against an empty external root failed with exit status 2 before it could validate any
record, proving that an unavailable source is never converted into a provenance pass:

```text
FAIL: cache input linux-virtio-gpu-uapi is unavailable
```

The closure suite passed 7/7 hermetic tests. Its synthetic full closure carries all 17 required source
families and a verified cache, then proves failure for a changed output hash, an unavailable input, and
a same-length SHA-256 mismatch. The remaining cases reject leaf, external-intermediate, and
internal-intermediate symlinks, non-regular cache entries, command drift, stale licenses, missing
sidecars, and an unreviewed sidecar.

## Final local gates

The inherited F02.2/F02.3 focused suites passed 15/15, 7/7, 7/7, 6/6, 6/6, 5/5, 10/10, 7/7, 3/3,
and 10/10; the new closure suite passed 7/7; source limits passed 6/6. `make test`, the roadmap
checker, and `git diff --check` passed locally after the completion markers. `make test` reported
1,151 Rust tests with zero failures (3 ignored) and 337 Node tests with zero failures, cancellations,
skips, or todos. macOS `xcrun` FSEvents/cache warnings occurred but did not affect either suite's
zero exit status.

This receipt closes provenance metadata and fresh source-byte availability only. It does not fetch into
Git, execute upstream payloads, implement VirGL/OpenGL/GLES, Venus/Vulkan, browser rendering, a
guest-visible GPU API, compatibility, native comparison, remote CI, or performance acceptance.
