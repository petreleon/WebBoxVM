# F03.2.3.1 evidence

Revision: `84c66f083dd6d37a3513d2199fbbe07ab77d4658`
Validation: focused hostile suite and final local project gate
Result: PASS
Artifacts: `opengl_limit_format_raw_inventory.json` SHA-256 `2289aa8603ec806e5d65398a8cff7e78318f629d964aa513c0bca5c560df84c6` (13,383 B)
Profile: OpenGL 4.6 core bounded raw limit/format facts; `matrix-incomplete`

Task ID and date: F03.2.3.1, 2026-09-21 Europe/Bucharest.

The source set became `84c66f08`. It verifies the external F02 PDF cache at
`/private/tmp/webboxvm-f0341.cqT6ZX`: `opengl-46-core-spec`, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`, 3,003,752 B, 851 pages.

It records 33 ordered raw facts from reviewed rows in tables 23.53–23.55 and 23.71, with exact physical
pages and numeric-section/table-row anchors. `coverage.complete=false`; shader, extension, state and
unreviewed table families remain visible exclusions. F03.2.3.4 prevents this table slice closing the full
core limit/format domain.

Commands from the repository root:

- `make graphics-opengl-limit-format-raw-inventory-test` — 5/5 passed.
- `make test` — exit 0; all focused new suites passed 30/30, Rust reported 1,130 cases (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — exit 0; pre-receipt structure was 441 documents / 265 tasks.

Hostile tests reject unadmitted GLSL/ESSL/registry/lower-profile substitutes, stale cache bytes,
missing/ambiguous table or row anchors, rehashed partial/reordered/duplicate/cross-profile data, forged
`complete=true`, invalid locators, promotion fields, and ambient import aliases. No guest, browser, GPU,
CTS, conformance, certification, or performance work ran. First failing subcheck: none.

Commit/push verification: `84c66f08` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run. Next ready work: F03.2.3.4.
