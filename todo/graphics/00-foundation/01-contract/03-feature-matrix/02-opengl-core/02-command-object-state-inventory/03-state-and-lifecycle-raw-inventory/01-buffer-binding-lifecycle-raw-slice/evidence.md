# F03.2.2.3.1 evidence

Revision: `84c66f083dd6d37a3513d2199fbbe07ab77d4658`
Validation: focused hostile suite and final local project gate
Result: PASS
Artifacts: `opengl_state_raw_inventory.json` SHA-256 `2a51dfda7b95ef377ecb383f3d2147b7d1b437f9eac451ec20f94867985129a9` (4,416 B)
Profile: OpenGL 4.6 core bounded buffer-binding raw slice; `matrix-incomplete`

Task ID and date: F03.2.2.3.1, 2026-09-21 Europe/Bucharest.

The exact retained external F02 PDF cache is `/private/tmp/webboxvm-f0341.cqT6ZX`; it verifies
`opengl-46-core-spec` SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`,
3,003,752 B, and 851 physical pages. The source set became revision `84c66f08`.

Three distinct raw facts are anchored: existing-buffer rebind and current-context deletion transitions on
physical page 84 §6.1, plus `ARRAY_BUFFER_BINDING` initial/query state on page 609 table 23.5 row 1.
`complete=false`: this does not form a complete state or transition grammar.

Commands from the repository root:

- `make graphics-opengl-state-raw-slice-test` — 8/8 passed.
- `make test` — exit 0; all focused new suites passed 30/30, Rust reported 1,130 cases (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — exit 0; pre-receipt structure was 441 documents / 265 tasks.

Hostile cases reject absent/ambiguous/reordered prose or table anchors, stale/mixed/symlinked caches,
duplicate/reordered/cross-profile facts, false support/owner/Matrix/CTS promotion, oversized or duplicate-key
manifests, and ambient import decoys. No guest, browser, GPU, CTS, conformance, certification, or
performance work ran; claims are false and Matrix/CTS counters are zero. First failing subcheck: none.

Commit/push verification: `84c66f08` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run. Next ready work: F03.2.2.3.2.
