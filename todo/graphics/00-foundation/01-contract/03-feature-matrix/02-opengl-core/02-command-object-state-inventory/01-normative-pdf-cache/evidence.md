# F03.2.2.1 evidence — OpenGL normative PDF cache

Revision: uncommitted shared worktree based on `6539a7dab4579eba971882d98899eba9f009d7d9`.
Validation: `make graphics-opengl-source-authority-test graphics-opengl-normative-pdf-cache-test`; `cargo test -p emulator --test source_file_limits --quiet`; `make test`; `git diff --check`; `python3 scripts/check_graphics_roadmap.py`
Result: PASS
Artifacts: `opengl_normative_pdf_cache.json` SHA-256 `3db3df111c46d238d34b09b11f6db0e82bdf1a810bdd30dd0814b26e3810957d`; embedded boundary self-hash `93cf3d3526ee1b5ddceee1034ae10543804b233083c3c45c43c6f667322c2101`
Profile: OpenGL 4.6 core cache boundary only; `blocked` / `matrix-incomplete`

Task ID and date: F03.2.2.1, 2026-09-21 Europe/Bucharest.

## Receipt

The reader reaches F02 only through the fixed, privately loaded F03.2.1 boundary. It accepts only its
`command-object-state` decision and the exact `opengl-46-core-spec` normative root: revision
`1cdd228e34966dd6b95bd203e9f84faba0f371a1`, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`, and 3,003,752 bytes.
No F02 manifest/v1 interface, `gl.xml`, GLSL, extension source, compatibility profile, or alternate root
is imported.

The reader accepts an absolute nonsymlink root outside this repository and only the identity-derived
`webboxvm-graphics/f02/opengl-46-core-spec/<sha256>.source` path. It opens a regular file without
following its final link, checks the exact byte count and SHA-256 before `pdfinfo`, and accepts exactly
851 physical one-based PDF pages. A locator must use F03.2.1's versioned grammar and lie in physical
page range 1 through 851; printed page labels are not substituted.

The retained `/private/tmp/webboxvm-f0341.cqT6ZX` cache was read only after these checks; no PDF was
copied into the repository. This is a source-byte boundary, not extraction or execution evidence:
all qualification claims are false, CTS executions and matrix rows are zero, and the profile remains
`blocked` / `matrix-incomplete`.

## Verification

- `make graphics-opengl-source-authority-test graphics-opengl-normative-pdf-cache-test` — PASS: 6 F03.2.1 and 6 F03.2.2.1 tests.
- The F03.2.2.1 hostile cases reject missing, stale, mixed, in-repository, symlinked, wrong-profile,
  alias/non-admitted, oversized, and invalid/out-of-range page inputs; its manifest asserts the no-claim receipt.
- CLI positive: the retained cache with `opengl46-core-pdf-v1:page=851;section=1` reports 851 physical
  pages and `matrix-incomplete`. Non-admitted class, page 852, and repository-root CLI probes each exit 2.
- `cargo test -p emulator --test source_file_limits --quiet` — PASS: 6 tests; each maintained file is at most 180 lines.
- `make test` — PASS: 1,130 Rust tests and 338 Node tests, with all graphics targets including this one.
- `git diff --check` and `python3 scripts/check_graphics_roadmap.py` — PASS before this receipt update;
  the roadmap reported 434 documents, 260 tasks, 68 PASS-complete, and 81 superseded.

`pdfinfo version 26.02.0` was the local page-count route. No guest image, browser session, renderer,
OpenGL call, CTS case, source extraction, owner mapping, matrix import, conformance/certification claim,
or performance measurement ran. No commit or push was requested. F03.2.2.2 may consume only this
verified raw byte source and remains unable to add a matrix row.
