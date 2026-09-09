# F06 evidence

Revision: F06.1 `464b34e`, F06.2 `56cda63`, F06.3 `86148af`
Validation: child receipts; source limits, `make test`, roadmap checker, and whitespace check
Result: PASS
Artifacts: browser-boundary split, maintained-source limit suite, generator/checker proof documents
Profile: structure and reproducibility maintenance only; no guest-visible graphics feature or performance claim

Task ID and date: F06, 2026-09-09.

F06.1 separated the browser graphics facade into maintained boundaries. F06.2 extended the
repository-wide 180-line source limit with reviewed exemptions. F06.3 added deterministic record
chunking, real-entrypoint roadmap checker fixtures, and repeatable CLI hash/staleness proof.

Child receipts: [F06.1](01-module-boundaries/evidence.md),
[F06.2](02-line-limit-coverage/evidence.md), and
[F06.3](03-generation-and-checker/evidence.md). Their focused checks and the aggregate source
limit, full-suite, roadmap, and whitespace commands passed from
`/Users/petreleon/code/WebBoxVM`.

This parent receipt covers code organization, bounded generated artifacts, and roadmap bookkeeping.
It does not establish guest protocol correctness, browser graphics behavior, Mesa, OpenGL/GLES,
Vulkan, WebGPU, native performance, or near-native performance. Commit/push verification: F06.3
implementation commit `86148af2d1a1e56969886e94e206b5536dfe8b27` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same SHA. No remote
CI result is claimed locally. Next ready task: F02.3.1.
