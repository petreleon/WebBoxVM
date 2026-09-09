# F02.3.3.4.4.2 evidence

Revision: `99a051b528859c871206a26b535899fe19cfbe9a` lock-consumer renewal
Validation: focused provenance/reproducibility suites, source limits, full local suite, roadmap, diff
Result: PASS
Artifacts: 11 F02.3 JSON sidecars and one F06 bundle bind raw lock `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`
Profile: lock-bound metadata and deterministic fixture renewal only; no WebIDL parser, API, guest, renderer, or performance behavior

Task ID and date: F02.3.3.4.4.2, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: `99a051b528859c871206a26b535899fe19cfbe9a`; clean before this receipt/status commit.
Upstream manifest revision: schema-v2 raw `inventory.lock` SHA-256
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.

## Renewal result

The renewal audit discovers every committed F02.3 JSON file carrying `inventory_sha256` and requires
its 11-path set to equal the reviewed map: six ABI sidecars, Venus, GL/GLES, two Vulkan/SPIR-V, and
the WGSL marker. Every record now names the new raw lock. Canonical non-lock JSON fingerprints freeze
IDs, input digests/licenses, commands, generator identity, artifact kind/path, and output hash. The
test replaces the new identity with the exact previous raw lock
`db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e` in every record; each fails
the generic contract with `stale inventory`.

F06 `input.json` changes only its lock identity. Deterministic regeneration changes its chunk header,
metadata lock, input digest, and chunk digest. The final input, chunk, and metadata SHA-256 values are
`0aa0cff6e734a9cc3afe11f0d49cccbab8df66012d2050f048a20313ffd4001a`,
`e6d0f0e680ef29a42005c5ce33343cfc88d67a29457e96473ebec296c12249c5`, and
`4e310c17a9aaecd0ad0f60b8815f0d021afc5a6eb9cd388146a79367a101b190`. The audit also gives a
temporary F06 input the exact old lock and observes nonzero `inventory lock identity does not match`.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`, Python 3.14 focused suites passed: F02.1 validator 6/6,
layout 6/6, F02.2 contract 15/15, generic provenance 10/10, ABI 7/7 plus direct 6-record check,
Venus 6/6, GL/GLES 6/6, Vulkan/SPIR-V 5/5, WGSL 10/10, renewal audit 3/3, chunker 9/9, and
reproducibility 4/4. The checked F06 bundle also passed `graphics_chunker.py --check` directly.
`cargo test -p emulator --test source_file_limits --quiet` passed 6/6. `make test` passed locally:
Rust 1,151 passed, 0 failed, 3 ignored; Node 337 passed, 0 failed. macOS `xcrun` cache-path warnings
were environmental warnings only. The roadmap checker passed 195 documents, 121 tasks, and 29 complete
before this receipt; `git diff --check` passed.

The exact first focused failure was `test_repeated_cli_generation_has_the_recorded_hashes`: its expected
F06 hashes still named the old lock-derived bundle. Updating only those two expected hashes made the
rereun pass. The first full-suite failure then occurred in the roadmap checker: historical receipts had
changed `Artifacts:` to an unsupported prefix. Restoring the required prefix while keeping the
tested-revision qualifier made the final full suite pass.

No payload was fetched, vendored, parsed, or compiled. No source identity or generator fixture payload
changed. This evidence does not establish WebGPU bindings, browser WebGPU, guest-visible VirGL/Venus,
compatibility, native comparison, or near-native performance.

Commit/push verification: feature commit `99a051b528859c871206a26b535899fe19cfbe9a` is local; this
receipt/status commit follows and both are pushed only after its structural checks. No remote CI result
is claimed.
Next ready task: F02.3.3.4.4.3 — Bind a WebGPU WebIDL generator record.
