# F02.2.1 evidence

Revision: `6c69d1b12ed80ec36d40988c201712cd1de63896` baseline; this scoped leaf is uncommitted
Validation: hermetic contract and inventory tests; full `make test`; final structural checks below
Result: PASS
Artifacts: no upstream payload was downloaded; the immutable inventory SHA-256 is
`8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e`
Profile: standard-library fetch/cache policy only; F02.2.2 fixtures and F02.2.3 live verification remain open

## Contract result

`source_model.py` accepts exactly the F02.1 schema and 15-family catalog, a unique matching entry
family set, canonical cache names, and only the raw GitHub or freedesktop GitLab HTTPS forms whose
raw path contains the exact 40-hex revision plus a file path. The caller must provide an absolute
cache root outside the repository; `$XDG_CACHE_HOME` in the manifest is not expanded by the verifier.

Before any cache write, the contract validates metadata, byte count and SHA-256. It denies redirects
and changed final URLs, re-hashes an existing entry before reuse, creates a fsynced temporary file in
the target directory, then atomically replaces the target. A failed validation leaves the expected
entry absent, while a corrupt existing entry is retained but never accepted.

## Commands and actual results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py
PYTHONDONTWRITEBYTECODE=1 make test
```

Actual: the contract suite ran 13 tests and passed all 13. It covers the committed 15-entry manifest,
verified external-cache placement, payload count/SHA mismatch, unsafe or mutable URLs, mismatched
revisions, missing/invented/duplicate source-family coverage, unsafe cache roots/names, malformed
revision/byte metadata/TOML, corrupt cache rejection, and redirect-handler denial. The inventory
parser printed `PASS: 15 immutable inputs` with the artifact SHA-256 above. `make test` passed 1,151
Rust tests with 0 failures and 3 ignored, plus 337 Node tests with 0 failures, cancellations, skips,
or todos.

The no-network cache-layout receipt printed:

```text
/private/tmp/webboxvm-f02-2-1-p3wtzngb/cache/webboxvm-graphics/f02/fixture/6996b93a2b8f3e4a22902c1116256ef5aa7165b5147719732473610b0ca9dda3.source
```

That temporary root was removed on process exit. The path proves the test-only payload was written
only below the caller-supplied external root and under its declared `local_cache` name.

## Boundary and handoff

No `source_fetch.py` live invocation occurred, no inventory payload was fetched, and no cache is
kept in the repository. F02.2.2 owns deterministic transport fixtures (including unavailable inputs
and redirect responses); F02.2.3 alone may claim a full live 15-input re-fetch and offline rehash.
After the completion markers were applied, the source-limit target passed 6/6, the roadmap checker
printed `PASS: 148 documents, 98 tasks, 6 complete; links/dependencies/limits valid` and `Ready:
F02.2.2, F06.3.2`, and `git diff --check` exited zero.
