# F02.2.2 evidence

Revision: `28b99a51013b0a6210ee1c51743d94be9c899e81` implementation commit; tests ran on its scoped worktree change
Validation: hermetic in-memory transport suite, inherited contract suite, full `make test`, and final checks below
Result: PASS
Artifacts: test-only 32-byte payload, SHA-256
`985b6e3e7172ecb3a4d72963dd3c84cbaf36ab6ae1efe0877891c15dc61d75b8`
Profile: isolated verification only; F02.2.3 live inventory verification remains open

## Fixture method

`fixture_transport.py` is a standard-library, in-memory fake: `FakeOpener.open()` either returns a
context-managed `FakeResponse` or raises an injected `URLError`/`HTTPError`. It records requested
URLs but does not create a listener, resolve a name, open a socket, or contact a public upstream.
The production F02.2.1 `fetch_to_cache(..., opener=...)` seam remains unchanged.

The success fixture has independently recorded constants: 32 bytes and the SHA-256 above. The suite
first asserts those constants against its literal payload, accepts it under the declared external
cache target, and then reuses/re-hashes it with an opener that would fail if called.

## Commands and actual results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/02-hermetic-fixtures/fixture_transport_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 make test
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

Actual before completion markers: the fixture suite ran 7 tests and passed all 7; F02.2.1's suite
ran 13 and passed all 13. The negative cases name and reject wrong SHA-256, mismatched revision
metadata, unavailable transport, malformed TOML, a 302 redirect, and a changed final URL. Each uses
a fresh temporary cache and asserts no target or cache root was created. `make test` passed 1,151
Rust tests with 0 failures and 3 ignored, plus 337 Node tests with 0 failures, cancellations, skips,
or todos. The source-limit target passed 6/6 and `git diff --check` exited zero.

No live source fetch occurred. F02.2.3 must independently fetch, record, and offline-rehash all 15
committed inputs; this receipt makes no claim about their present public availability or bytes.

## Final structural result

After the completion markers were applied, the source-limit target passed 6/6, the roadmap checker
printed `PASS: 150 documents, 98 tasks, 8 complete; links/dependencies/limits valid` and `Ready:
F02.2.3, F06.3.3`, and `git diff --check` exited zero.

Commit/push verification: implementation commit
`28b99a51013b0a6210ee1c51743d94be9c899e81` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same SHA. No remote
CI result is claimed locally. Next ready task: F02.2.3, which needs a separately authorized live
fetch and offline re-hash of all 15 pinned inputs.
