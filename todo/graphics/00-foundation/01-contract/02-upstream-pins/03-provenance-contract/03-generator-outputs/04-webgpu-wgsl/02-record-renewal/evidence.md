# F02.3.3.4.2 evidence

Revision: 74ff981f80e94189e373b548bf14b9f7a8ebb430
Validation: affected offline suites + identity/stale-lock audit + source limits + full local suite
Result: PASS
Artifacts: at the tested revision, canonical raw lock `db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e`; ten F02.3 sidecars and one F06 expected bundle renewed
Profile: provenance/reproducibility record renewal only; no fetch, guest, API, browser, renderer, or performance behavior

Task ID and date: F02.3.3.4.2, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: 74ff981f80e94189e373b548bf14b9f7a8ebb430; clean before this
receipt. The separate documentation/status receipt follows this implementation commit.
Upstream grammar revision and retrieval: unchanged from F02.3.3.4.1.2; this task fetched no payload and
does not repeat or extend its source evidence.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.

## Renewal result

At this receipt's tested revision, the raw `inventory.lock` SHA-256 was recomputed as
`db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e`.
All six ABI sidecars, the Venus sidecar, the GL/GLES sidecar, and both Vulkan/SPIR-V sidecars now
then bound that one identity. The contemporaneous audit compared each JSON record to its pre-renewal
version: every field except `inventory_sha256` is byte-for-byte structurally unchanged, including IDs,
input digests, licenses, commands, generator versions, artifact paths, kinds, and output hashes.

At that revision, F06 input records were unchanged except for the lock identity. Deterministic regeneration changed only
lock-derived output fields: the chunk header, metadata lock, metadata input hash, and metadata chunk
hash. Its hashes then were input `35d74836833744aa9199fb885e6f4446cb4501b8a52a53885277374abbedab26`,
chunk `8c7ccca58afacacdebdac8571fe766d719c470a05b80aa682efceba25d85066e`, and metadata
`dfaae1b2ec3df5fc7cc384b085ca78a11e7fcc725a761f9d22243ceae6ef29e9`.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`, Python 3.14 hermetic suites passed: layout 5/5, F02.1 6/6,
F02.2 source contract 15/15, transport 7/7, provenance 10/10, ABI 7/7, Venus 6/6, GL/GLES 6/6,
Vulkan/SPIR-V 5/5, WebGPU boundary 7/7, chunker 9/9, and reproducibility 4/4. Direct ABI and GL
validators passed. `cargo test -p emulator --test source_file_limits --quiet` passed 6/6. `make test`
passed locally with Rust 1,151 passed, 0 failed, 3 ignored and Node 337 passed, 0 failed.
`python3 scripts/check_graphics_roadmap.py` reported 188 documents, 117 tasks, and 26 complete before
this receipt; `git diff --check` passed.

The old raw lock `cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b` was substituted
into every renewed F02.3 record and each failed with `record has a stale inventory_sha256`. A temporary
F06 specification using it failed with `inventory lock identity does not match chunk specification`.
Thus an old lock cannot silently validate a renewed record or bundle.

First failing subcheck or blocker: the first whole-suite attempt reached the roadmap checker and failed
because ignored generated `__pycache__/inventory_layout.cpython-314.pyc` was decoded as roadmap text.
Removing only that generated cache made the unchanged `make test` and roadmap check pass; no source or
task artifact was changed to address it. The renewal-focused suites themselves had no failure.

Decision and limits: accept only lock-bound metadata renewal. No source identity, payload byte,
generated fixture payload, graphics behavior, WebGPU/VirGL/Vulkan API, or runtime result changes here.
At that revision the WGSL grammar was not a WebGPU generator input, so F02.3.3.4.4 remained BLOCKED. This is not a WGSL
parser/compiler, a guest integration, a renderer, or evidence of compatibility or near-native speed.
No remote CI was run. The local commit is not pushed because authorization for
`https://github.com/petreleon/WebBoxVM.git` has not been granted.

Next ready task: F02.3.3.4.3 — Bind a WGSL grammar generator record.
