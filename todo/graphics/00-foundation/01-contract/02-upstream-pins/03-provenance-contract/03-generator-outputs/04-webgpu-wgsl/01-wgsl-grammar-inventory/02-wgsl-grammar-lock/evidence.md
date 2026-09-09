# F02.3.3.4.1.2 evidence

Revision: fefc28123ba70dae24f8a8a640c94976ceddc632
Validation: live 16/16 fetch + offline 16/16 rehash + focused 40/40 + boundary 7/7 + limits + full suite
Result: PASS
Artifacts: grammar sha256=838b6fd1d01e4efd06e233200479d57667e8f8ba74783598e51f8f6195f762a1; root sha256=22e5e250b3d475ef2607b2bc92b9885e2910824273d8acb7be19fc6614fc8cda; lock sha256=db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e
Profile: immutable-source integrity and grammar classification only; no parser, compiler, WebGPU API, guest, browser, or graphics-runtime behavior

Task ID and date: F02.3.3.4.1.2, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: fefc28123ba70dae24f8a8a640c94976ceddc632; clean tree before this receipt.
Upstream manifest revision: schema-v2 raw lock SHA-256
db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact command(s), working directory and tool versions: Python 3.14 from
`/Users/petreleon/code/WebBoxVM`: `inventory_layout_test.py`; `validate_manifest.py --self-test`;
`validate_manifest.py`; `source_fetch_test.py`; `fixture_transport_test.py`;
`webgpu_generator_boundary_test.py`; two `source_fetch.py --cache-root` runs against a newly made
external cache; then a local SHA-256/byte-count audit, `make test`,
`cargo test -p emulator --test source_file_limits --quiet`,
`python3 scripts/check_graphics_roadmap.py`, and `git diff --check`.
Expected result and minimum nonzero case count: exactly one new grammar-family source retains immutable
URL/commit/bytes/hash/license facts, all 16 sources fetch then offline-rehash, the grammar and semantic
WGSL reference remain distinct, and the WebGPU boundary stays blocked.
Actual passed/failed/skipped counts and exit codes: initial fetch 16/0 unavailable; offline reuse and
rehash 16/0 unavailable; independent cache audit 16/0 mismatch; layout 5/0/0; F02.1 6/0/0;
F02.2 contract 15/0/0; transport 7/0/0; boundary 7/0/0 (40/0/0 focused); limits 6/0/0; full Rust
1,151/0/3 ignored; Node 337/0/0; all required commands exited 0.
Negative/reference checks and observed output: the layout/validator reject malformed, missing-family,
mutable, stale-lock, and source-role confusion fixtures. The real grammar entry is rejected as
WGSL-derived by the WebGPU boundary; the probe reports only reference-only `webgpu-spec`, WGSL-derived
`wgsl-spec`, and no explicit WebGPU generator input.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the fresh cache was
`/private/tmp/webboxvm-f02-wgsl-live.E64MYf`, contained 16 verified files, and was removed after the
audit; the separate raw-source/license inspection cache was also removed. Recreate with the two listed
fetch commands and the immutable URL in `inputs/part-0001.toml`; no payload is tracked in Git.
Software fallback detection and actual execution route: direct pinned HTTPS fetch followed by stdlib
byte-count/SHA-256 verification; no parser fallback, renderer, or GPU route.
Performance conditions and frozen protocol version, when applicable: not applicable; none measured.
First failing subcheck or blocker, when applicable: no required subcheck failed. Deliberate downstream
handoff check: the first F02.3 ABI validator fails `record has a stale inventory_sha256`; the F06 proof
then fails because its fixture still names the prior lock. F02.3.3.4.2 owns their atomic renewal.
Decision and limits of the evidence: accept only the grammar source identity and narrow dialect role.
`syntax.bnf` says it is not directly compatible with existing BNF parsers and uses pattern literals plus
generalized RegEx operations; it does not establish a WGSL compiler, WebGPU API, guest rendering, or
near-native performance.
Commit/push verification: local feature commit fefc28123ba70dae24f8a8a640c94976ceddc632; not pushed
because explicit authorization for `https://github.com/petreleon/WebBoxVM.git` has not been granted.
Next ready task: F02.3.3.4.2 — Renew lock-bound records and reproducibility proof.
