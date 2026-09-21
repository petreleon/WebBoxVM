# F03.2.2.2 evidence

Revision: `84c66f083dd6d37a3513d2199fbbe07ab77d4658`
Validation: focused hostile suite and final local project gate
Result: PASS
Artifacts: `opengl_command_object_raw_inventory.json` SHA-256 `3720199fb4a69bacc90e8187b3fcdda5cbdb67bc12cc3e83f3cd230ba6edd0ee` (12,342 B)
Profile: OpenGL 4.6 core raw-source planning only; `matrix-incomplete`

Task ID and date: F03.2.2.2, 2026-09-21 Europe/Bucharest.

The tested source set became the revision above. It binds the retained external F02 PDF cache
`/private/tmp/webboxvm-f0341.cqT6ZX`, exact source `opengl-46-core-spec`
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee` (3,003,752 B, 851
physical pages). The cache is external and reproducible from the sealed F02 identity.

The inventory contains 24 raw facts: the 12 §2.6 object families plus 12 formal direct-creation
declarations. Its scope is explicitly limited to those families; it does not claim the full OpenGL command
universe. F03.2.2.5 remains open for that vocabulary.

Commands from the repository root:

- `make graphics-opengl-command-object-raw-inventory-test` — 5/5 passed.
- `make test` — exit 0; all focused new suites passed 30/30, Rust reported 1,130 cases (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — exit 0; pre-receipt structure was 441 documents / 265 tasks.

Negative checks reject stale/mixed or symlinked caches, altered source anchors, duplicate/reordered/partial
facts, cross-profile data, false support, Matrix ingress, and ambient import aliases. No guest, browser,
GPU, CTS, native comparison, or performance command ran; all claims are false and Matrix/CTS counters are
zero. First failing subcheck: none.

Commit/push verification: `84c66f08` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run. Next ready work: F03.2.2.3.2 and F03.2.2.5.
