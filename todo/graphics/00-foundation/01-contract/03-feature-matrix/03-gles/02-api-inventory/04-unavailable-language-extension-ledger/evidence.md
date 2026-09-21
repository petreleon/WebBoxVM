# F03.3.2.4 evidence — unavailable GLES language and extension ledger

Revision: uncommitted worktree based on `6539a7dab4579eba971882d98899eba9f009d7d9`.
Validation: `make graphics-gles-source-authority-test graphics-gles-unavailable-ledger-test`; `cargo test -p emulator --test source_file_limits --quiet`; `make test`; `git diff --check`; `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py`
Result: PASS
Artifacts: [`gles_unavailable_ledger.json`](gles_unavailable_ledger.json) SHA-256 `535e9793f3d4733e65c84ae61c681e116dce93131d2386228df90c5176dd2979`; embedded ledger SHA-256 `560de6a8c6d8102fe83e3009dd57b7558b0e86029f5e137cb8fe467f85677ea9`
Profile: GLES 3.2 source-decision ledger only; `blocked` / `matrix-incomplete`; zero Matrix rows and CTS executions

Task ID and date: F03.3.2.4, 2026-09-21 Europe/Bucharest.

## Receipt

The self-hashed ledger loads F03.3.1 through a fixed private path and binds the active role-aware source
contract `d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3` and inventory lock
`44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`. It retains the exact GLES
normative root `gles-32-spec` and full-suite root `gles-cts-main` identities.

`shader`, `precision`, and `extension` retain exactly `unadmitted-distinct-source`: the GLES PDF points
language/precision semantics to the separately unadmitted ESSL source. ESSL aliases, `gl.xml`, desktop
GLSL, lower GLES/ESSL versions, stale or mixed identities, reordered decisions, and promotions fail
closed. The local `reject_matrix_row` API rejects each unavailable class even with `blocked` status; a
nonblocked status is rejected as a promotion. It is not a global Matrix v2 hook; F03.3.2.5 remains the
handoff boundary for later GLES inventory import.

All six qualification claims are false. No matrix row, CTS case, guest, browser, renderer, certification,
or performance result was produced.

## Verification

- F03.3.1 source authority: 6 focused tests passed; ledger: 5 focused positive/hostile tests passed.
- `cargo test -p emulator --test source_file_limits --quiet` — 6 tests passed.
- `make test` — passed: graphics checks, 1,127 Rust tests (3 ignored), and 338 Node tests.
- `git diff --check` and the graphics-roadmap checker passed before this receipt update.
