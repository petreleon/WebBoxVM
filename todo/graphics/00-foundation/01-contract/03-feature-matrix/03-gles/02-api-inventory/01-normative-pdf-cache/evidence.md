# F03.3.2.1 evidence

Revision: `1a0a2c5755a6dcbc4951adf823c713126ca868dd`
Validation: focused hostile suite, retained-cache replay, and final local project gate
Result: PASS
Artifacts: `gles_normative_pdf_cache.json` SHA-256 `0e695dbf47abfa43a6358ee6cca24013828c7ff035eb4d344a712d9f9e2ae9fd`
Profile: GLES 3.2 normative-PDF cache boundary; `matrix-incomplete`

Task ID and date: F03.3.2.1, 2026-09-21 Europe/Bucharest.

The exact source is the retained external F02 cache `/private/tmp/webboxvm-f0341.cqT6ZX`, record
`gles-32-spec`, revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`, SHA-256
`5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`, 2,198,754 B, and 601
physical PDF pages. It accepts only `command-state` and `limit-format` through the fixed F03.3.1 boundary.

The self-hashed cache manifest has boundary SHA-256
`c32574099c24c6d6b78b43ef82cf88869853353cdac619ca9f0b8b58100b5f79`. It rejects internal or symlinked
roots, symlinked intermediate paths, nonregular/FIFO payloads, stale or oversized bytes, wrong profiles,
changed inodes, absent `O_NOFOLLOW` or `O_NONBLOCK`, bad digest/page count, and rehashed claim promotion.

Commands from the repository root:

- `make graphics-gles-normative-pdf-cache-test` — 7/7 passed without a cache skip.
- `PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/01-normative-pdf-cache/gles_normative_pdf_cache.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX --locator 'gles32-pdf-v1:page=601;section=1' --source-class command-state` — `PASS: GLES normative PDF 601 physical pages; matrix-incomplete`.
- `make test` — exit 0; new focused source suites passed 37/37, Rust reported 1,130 tests (3 ignored), and Node reported 338 pass / 0 fail.
- `cargo test -p emulator --test source_file_limits --quiet` — 6/6 passed; `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` also passed.

No API inventory, Matrix row, owner, CTS run, guest/browser/GPU behavior, conformance, certification, or
performance result exists. `matrix_rows` and `cts_executions` remain zero and every claim is false. First
failing subcheck: none. Next ready work: F03.3.2.2 and F03.3.2.3.

Commit/push verification: `1a0a2c57` was pushed to `codex/graphics-f01-baseline` and `git ls-remote`
matched it. `gh run list --commit` returned no run.
