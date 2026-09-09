# F02.3.3.4.1.1.1 evidence

Revision: 2e4969f2b317530906ee52494ba84f0f30367f58
Validation: focused 5/5 + existing 4/4 + source limit 6/6 + roadmap + make test
Result: PASS
Artifacts: `inventory_layout.py` sha256=dd9cadbb9baf4e655c1bfccfb698aa60557b8dbb6b4366f8d1a2a7c6cdf616d4; `inventory_layout_test.py` sha256=5c4daaa6b238b939d4eebfa652f8a27a10c8e7898a38012c6272333263044e7b
Profile: provenance-layout only; no upstream payload, guest, API, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1.1.1, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: 2e4969f2b317530906ee52494ba84f0f30367f58; clean tree before receipt.
Historical pre-cutover manifest revision: schema-v1 raw SHA-256
8f81ece8dc895698f2715493c927b3f5ad10f24a9f20f1b48d82405a4a9a288e.
Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task has no browser or GPU execution path.
Exact command(s), working directory and tool versions:
`PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/inventory_layout_test.py`; `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test`; `cargo test -p emulator --test source_file_limits --quiet`; `python3 scripts/check_graphics_roadmap.py`; `git diff --check`; `make test`, from `/Users/petreleon/code/WebBoxVM`.
Expected result and minimum nonzero case count: the new hostile-fixture suite has at least one
passing case; altered root, component, lock, layout paths, fragments, and symlinks fail closed.
Actual passed/failed/skipped counts and exit codes: layout 5/0/0; existing F02.1 4/0/0; source
limit 6/0/0; `make test` Rust 1,151 passed, 0 failed, 3 ignored; Node 337 passed, 0 failed; all 0.
Negative/reference checks and observed output: missing/changed root, component, or lock; unsafe,
duplicate, unordered, or omitted component names; malformed fragments; and intermediate/lock
symlinks raise `InventoryLayoutError`. The same v2 fixture in two locations has one lock identity.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external sample,
image, or payload was retained; recreate the five local temporary fixtures with the focused command.
Software fallback detection and actual execution route: not applicable; Python stdlib parser only.
Performance conditions and frozen protocol version, when applicable: not applicable; no performance
or protocol behavior is measured.
First failing subcheck or blocker, when applicable: the first roadmap run found a generated
`01-input-inventory/__pycache__/inventory_layout.cpython-314.pyc` from an earlier local test; it was
removed as a temporary artifact, then the checker and full suite passed. No implementation test failed.
Decision and limits of the evidence: accept only the layout/lock contract. At this receipt's revision,
the checked inventory stayed schema v1; no source was added, fetched, or reclassified, and no real
graphics feature was claimed.
Commit/push verification: local feature commit 2e4969f2b317530906ee52494ba84f0f30367f58; not pushed
because explicit authorization for the remote destination has not been granted.
Next ready task: F02.3.3.4.1.1.2, with F02.3.3.4.1.1.3 and F02.3.3.4.1.1.4 independently ready after
this completion.
