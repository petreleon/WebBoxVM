# F06.3 evidence

Revision: F06.3.1 `6469074`, F06.3.2 `8e04cc9`, F06.3.3 `86148af`
Validation: all three child receipts; generator CLI, checker fixtures, source limits, `make test`, roadmap, diff
Result: PASS
Artifacts: chunk generator, hermetic checker fixtures, and CLI reproducibility proof linked below
Profile: generated-record and roadmap-structure maintenance only; no guest graphics behavior or performance claim

Task ID and date: F06.3, 2026-09-09.

F06.3.1 introduced deterministic, provenance-bound record chunking with a 180-line maximum.
F06.3.2 exercises the real roadmap checker with a valid nested root and four deterministic
structural failures. F06.3.3 runs the checked input through the generator CLI twice in fresh
directories, records equal chunk and metadata hashes, rejects stale metadata and reordered input,
and reruns the F06.3.2 fixture suite.

Child receipts: [F06.3.1](01-chunk-generator/evidence.md),
[F06.3.2](02-checker-fixtures/evidence.md), and
[F06.3.3](03-reproducibility-proof/evidence.md). The grouped acceptance checks were run from
`/Users/petreleon/code/WebBoxVM`; each child records its focused command counts and hash values.
The aggregate source-limit, roadmap, whitespace, and full-suite commands exited zero.

This aggregation proves maintenance-tool determinism and structural checking, not graphics protocol
generation, browser execution, API compatibility, native comparison, fallback behavior, Mesa,
OpenGL/GLES, Vulkan, WebGPU, or performance. Commit/push verification: F06.3.3 implementation
commit `86148af2d1a1e56969886e94e206b5536dfe8b27` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same SHA. No remote
CI result is claimed locally. Next ready task: F02.3.1.
