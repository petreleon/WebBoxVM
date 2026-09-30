# I01 bounded native command runner checkpoint

Baseline: `d643e6106034df498bb26a54c30fed19808b09c6`.
Implementation: `8b9bb4af` (all 12 per-file digests match the tested frozen runner).
Scope: host command framing, finite VM execution and raw UART capture.
I01 remains open; this checkpoint does not certify a stock-Mesa image or startup.

## Implemented behavior

`emulator/examples/guest_command.rs` accepts an explicit ARM64 kernel, initramfs,
one to sixteen command files, optional WBDISK and exact required output lines.
It boots through the existing BootContext API and injects encoded shell scripts
after an exact readiness line. The guest must return ordered zero command statuses
and a separate final control-shell round trip. Assertions count only complete
lines inside successful command execution, never boot text or echoed scripts.

The default bounds are unchanged: 900 seconds, 20 billion instructions,
2 million per chunk, 1 MiB UART and 128 KiB injected input. CLI overrides may
reduce them. Even zero-step VM calls exhaust a finite chunk budget. Guest panic,
Oops, malformed or duplicate framing, invalid UART encoding, nonzero status,
missing assertions and exhausted bounds fail. UART files use create_new and
preserve raw bytes, including output from a failing final chunk.

No host GPU completion, CPU pixel oracle or stock-Mesa success is synthesized.

## Observed verification

- `cargo test -p emulator --example guest_command`: 25 PASS. Includes real host
  shell round trips for arbitrary script text, errexit, decoder failure and closed
  child stdin, plus hostile protocol chunks, assertion scope and finite budgets.
- `cargo test -p emulator --test source_file_limits`: 6 PASS.
- Independent read-only protocol review: no remaining actionable issue after
  doubled-CR assertions, pending action guards and final DONE round trip fixes.
- `make -j4 test`: exit 0; 566 Python tests in 84 suites, 1198 Rust tests,
  3 pre-existing ignored Rust tests, 342 web tests; 738.71 seconds. This includes
  this frozen runner plus the then-current fixture preparation. Later fixture
  corrections require their own focused checks and source-limit validation.

Full log: `.artifacts/graphics/i01-mesa-image/integration-full-test-2026-09-30.log`.
SHA256: `779bc6f89f0e602396c1f5eac73c961f5ff1df5fa7cc59978ef3d7272a19d330`.

The 12 runner files have sorted repository-relative UTF-8 path+NUL+ASCII
per-file-SHA256+LF fingerprint
`35f03d4000a170bb11586bf2d30f8f525ef71b3d02561a1d853d871c7fd85cfc`.
The initial aggregate supplied by the worker did not reproduce under this
explicit serialization; every individual source digest matched, and the
aggregate was recalculated directly before publication.
The largest file is 171 physical lines. Dedicated example tests are wired into
the fixture test target being prepared; ordinary Cargo package tests alone do
not execute these example tests.

## Remaining real lane

Run this exact runner against the assembled stock-Mesa fixture, retain UART and
the first failure/partial effects, and register the task through F05. A clean
image reproduction and actual renderer/ICD report are still required by I01.
