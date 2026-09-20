# F02.5.4.3 evidence

Revision: `f5337681`
Validation: nine focused adapter/receipt tests, fresh nine-file selector cache, active F03 gate and
schema-boundary probe, full local suite, source-file limits, diff, and roadmap checks
Result: PASS
Artifacts: [self-hashed aggregate receipt](f05_source_aggregate_receipt.json), SHA-256
`85980a92e5223261e7fce6fc162e10171bd671800a5fe9d4c2460e01b68d5242`; external cache
`/private/tmp/webboxvm-f02543.bO41B6` (nine regular files)
Profile: source provenance only; no profile registration, guest, browser, renderer, or CTS result

Task ID and date: F02.5.4.3, 2026-09-20 Europe/Bucharest. Tested code commit: `f5337681`.
The accessor accepts the F02.5.4.1 raw-byte seal only and returns all eight source records, six ordered
mandatory bindings, three full-suite closures, and four no-claim auxiliary entries. It has no profile
argument and cannot register an F05 check. F05.1 and `scripts/graphics/` were not changed.

## Fresh capture and downstream checks

The aggregate generator created the new external root above from an empty directory. It fetched and
verified exactly seven selector records and two release-license proofs; the largest selector remains the
3,309,653-byte registry. The cache contains no VCTS `api.txt`, VCTS closure member, guest image, or CTS
binary. The aggregate receipt binds source-contract SHA-256
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3` and inventory-lock SHA-256
`44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`.

The active F03 gate passed and still reports every profile `blocked` by `matrix-incomplete`. A temporary
three-row all-blocked schema fixture tested role resolution only; it imported zero real matrix rows and
is not a coverage or support result. The receipt keeps all six qualification claims false and CTS
executions at zero.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

```sh
make graphics-f05-source-adapter-test
# adapter: 5 passed; aggregate receipt: 4 passed
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_aggregate_receipt.py \
  --selector-cache-root /private/tmp/webboxvm-f02543.bO41B6 --timeout 60
# fresh selector cache, F03 gate, schema-boundary probe, and receipt: exit 0
PYTHONDONTWRITEBYTECODE=1 python3 \
  todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_aggregate_receipt.py \
  --selector-cache-root /private/tmp/webboxvm-f02543.bO41B6 \
  --receipt todo/graphics/00-foundation/01-contract/02-upstream-pins/05-source-role-admission/04-inventory-and-consumer-cutover/03-future-f05-adapter-and-receipt/f05_source_aggregate_receipt.json
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

`make test` passed its graphics, 1,127-pass/3-ignored Rust, and Node suites; source-file limits passed
6/6. Negative tests reject raw-lock reformatting and symlinks, missing/reordered roles or closures,
alias/mixed/auxiliary substitutes, promoted claims or CTS count, stale self-hashes, cache evidence
changes, matrix promotion, profile registration, and ambient-module decoys while restoring `sys.path`.

Guest image and build hashes; browser, OS, adapter, driver, software fallback, and performance protocol:
not applicable because no guest/browser/renderer path ran. There is no performance or near-native claim.
Commit/push verification: `f5337681` is pushed to `origin/codex/graphics-f01-baseline`; `gh run list`
returned no matching Actions run. Next ready task: F03.2, F03.3, or F03.4.1.
