# F02.3.3.4.1 evidence

Revision: fefc28123ba70dae24f8a8a640c94976ceddc632
Validation: composite-lock receipts + grammar 16/16 fetch/reuse audit + focused 40/40 + limits + full suite
Result: PASS
Artifacts: at the tested revision, schema-v2 root sha256=22e5e250b3d475ef2607b2bc92b9885e2910824273d8acb7be19fc6614fc8cda; component sha256=eef0b188e9e663e6be483bfedde5df27687db9b590ff2bd208ebf462b0cc6649; lock sha256=db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e
Profile: immutable WGSL grammar provenance only; no parser, compiler, WebGPU API, guest, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: fefc28123ba70dae24f8a8a640c94976ceddc632; clean tree before receipts.
Upstream manifest revision at the tested commit: schema-v2 raw lock SHA-256
db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact command(s), working directory and tool versions: the completed composite-layout child receipts
plus the grammar leaf's Python 3.14 layout, F02.1, F02.2, boundary, live-fetch, offline-audit,
source-limit, `make test`, roadmap, and diff commands, all from `/Users/petreleon/code/WebBoxVM`.
Expected result and minimum nonzero case count: bounded components preserve the original 15 identities,
one distinct grammar entry forms the sixteenth family, the raw lock binds the closure, and all 16 pinned
payloads fetch/reuse with nonzero focused coverage.
Actual passed/failed/skipped counts and exit codes: grammar leaf fetch/reuse 16/16; focused 40/0/0;
source limits 6/0/0; full Rust 1,151/0/3 ignored; Node 337/0/0; all required commands exit 0.
Negative/reference checks and observed output: layout rejects closure changes; the grammar/source-role
test separates `wgsl-spec` from `wgsl-grammar-syntax`; the actual grammar cannot be a WebGPU generator
input. The changed lock intentionally makes old F02.3/F06 records stale until F02.3.3.4.2 renews them.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: external fetch and raw
inspection caches were removed after their audits. Recreate from the locked URLs and commands in the
grammar leaf receipt; no upstream payload is stored in Git.
Software fallback detection and actual execution route: direct HTTPS cache verification and Python
stdlib layout checks only; no parser, rendering, or GPU route.
Performance conditions and frozen protocol version, when applicable: not applicable; none measured.
First failing subcheck or blocker, when applicable: no required check failed; the next leaf owns known
stale lock-bound F02.3/F06 data.
Decision and limits of the evidence: accept the lock and grammar inventory only. This does not prove
WGSL compilation, WebGPU support, guest-visible graphics, or near-native performance.
Commit/push verification: local feature commit fefc28123ba70dae24f8a8a640c94976ceddc632; not pushed
because explicit authorization for `https://github.com/petreleon/WebBoxVM.git` has not been granted.
Next ready task: F02.3.3.4.2 — Renew lock-bound records and reproducibility proof.
