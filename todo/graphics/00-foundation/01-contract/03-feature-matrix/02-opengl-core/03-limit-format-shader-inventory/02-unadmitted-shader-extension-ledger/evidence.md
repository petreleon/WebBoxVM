# F03.2.3.2 evidence

Revision: `84c66f083dd6d37a3513d2199fbbe07ab77d4658`
Validation: focused hostile suite and final local project gate
Result: PASS
Artifacts: `opengl_unadmitted_ledger.json` SHA-256 `a6e91aead8dde1e21357e5cd34d53d52b96d03df51295f171dad186c813f1548` (2,777 B)
Profile: OpenGL 4.6 core source-policy ledger; `matrix-incomplete`

Task ID and date: F03.2.3.2, 2026-09-21 Europe/Bucharest.

The source set became `84c66f08`. The self-hashed ledger pins F03.2.1 boundary
`71d1fdbe2bb51911d6fbf6fe4a5c5665681c5a4a8a265b3db840bc629762a641`, F02 source contract
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3`, and inventory lock
`44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`.

It records exactly two unavailable classes, `shader` and `extension`, with zero semantic facts, Matrix
rows, CTS executions, and claims. It does not admit GLSL, ESSL, `gl.xml`, a registry alias, a lower
profile, or a compatibility profile as substitute authority.

Commands from the repository root:

- `make graphics-opengl-unadmitted-ledger-test` — 6/6 passed.
- `make test` — exit 0; all focused new suites passed 30/30, Rust reported 1,130 cases (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — exit 0; pre-receipt structure was 441 documents / 265 tasks.

Negative checks reject stale/rehashed/mixed/reordered/promoted or wrong-typed ledgers, stale contract/lock
or authority inputs, source aliases, symlink/missing authority, Matrix ingress, and direct substitute
requests. No shader execution, guest/browser path, GPU, CTS, conformance, certification, or performance
work ran. First failing subcheck: none.

Commit/push verification: `84c66f08` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run. Next ready work: F03.2.3.3 after F03.2.3.4.
