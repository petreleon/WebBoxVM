# F02.3.3.4.3 evidence

Revision: 0e36b01a241bc83d3176e2821a6076e13756da2d
Validation: direct fixture check + hermetic 10/10 + provenance/inventory/boundary suites + limits + full local suite
Result: PASS
Artifacts: at the tested revision, fixture sha256=605721be432ffe189c104292c173597399236c1e226f00d5d1f5d2e1c24e5848; sidecar sha256=43fed69da001254b05dfbe2c99427c60b8d6d3e9a7016d75dc66ad63dff39e41
Profile: fixture-only WGSL grammar provenance marker; no grammar bytes, parser, compiler, WebGPU API, guest, browser, renderer, or performance behavior

Task ID and date: F02.3.3.4.3, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: 0e36b01a241bc83d3176e2821a6076e13756da2d; clean before this
receipt. The separate documentation/status receipt follows this implementation commit.
Upstream manifest revision at the tested commit: schema-v2 raw `inventory.lock` SHA-256
`db22bb053108cb7dbd413d4ee221a8134c081c842ead538e4d5fe0c45789e75e`; only
`wgsl-grammar-syntax` is bound, with SHA-256
`838b6fd1d01e4efd06e233200479d57667e8f8ba74783598e51f8f6195f762a1` and the reviewed
`W3C Software and Document License (repo LICENSE.md; document)` label.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.

## Fixture result

`generate_fixture.py` reads the lock-validated manifest identity only and writes a 13-line JSON marker.
It freezes the `wgsl-grammar` family, the exact grammar role, digest, license, generator
`f02-wgsl-grammar-fixture` version `1`, and the limitation: nonstandard BNF dialect only, neither
parsed nor compiled. The marker contains no upstream grammar payload and no WebGPU API information.
The schema-v2 sidecar freezes its command, artifact path, lock identity, output hash, and sole input.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`, Python 3.14 commands passed: `validate_manifest.py --self-test`
6/6; `source_fetch_test.py` 15/15; `provenance_record_test.py` 10/10; the new direct
`validate_fixture.py` check; new `validate_fixture_test.py` 10/10; and
`webgpu_generator_boundary_test.py` 7/7. `cargo test -p emulator --test source_file_limits --quiet`
passed 6/6. `make test` passed locally with Rust 1,151 passed, 0 failed, 3 ignored and Node 337
passed, 0 failed. `python3 scripts/check_graphics_roadmap.py` and `git diff --check` passed.

The hermetic fixture suite proves byte-identical regeneration from the manifest without network and
rejects `wgsl-spec`, `webgpu-spec`, and `opengl-gles-registry`; stale lock, digest, and license;
wrong generator version and command; changed family/role; altered output bytes; a false declared hash;
and a tampered grammar-dialect limitation even when its replacement hash is declared. At that revision,
the real WebGPU boundary printed: reference-only `webgpu-spec`, WGSL-derived `wgsl-spec`, and no
explicit WebGPU generator input.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: the tracked fixture and
sidecar hashes are above. Reproduce from this task folder with
`PYTHONDONTWRITEBYTECODE=1 python3 generate_fixture.py --manifest ../../../../01-input-inventory/manifest.toml --output fixture-output.json`,
then run `PYTHONDONTWRITEBYTECODE=1 python3 validate_fixture.py` and
`PYTHONDONTWRITEBYTECODE=1 python3 validate_fixture_test.py`. No external cache, raw payload, image,
or runtime capture was created or retained.
Software fallback detection and actual execution route: Python stdlib JSON/SHA-256 and local manifest
validation only; no source fetch, parser fallback, renderer, GPU, guest, or browser route.
Performance conditions and frozen protocol version, when applicable: not applicable; none measured.
First failing subcheck or blocker, when applicable: a deliberately parallel initial boundary test raced
the F02.1 self-test while both copied/deleted an inventory temporary directory. Its isolated rerun
passed 7/7; no implementation or fixture behavior failed. The remaining WebGPU-input boundary is an
intentional downstream BLOCKED condition, not a passing WebGPU feature.
Decision and limits of the evidence: accept one provenance fixture only. It does not establish a WGSL
parser/compiler, WebGPU binding, guest-visible API, rendering, source-fetch/cache proof, compatibility,
or near-native performance. No remote CI was run.
Commit/push verification: local feature commit 0e36b01a241bc83d3176e2821a6076e13756da2d; not pushed
because explicit authorization for `https://github.com/petreleon/WebBoxVM.git` has not been granted.
Next ready task at that revision: F02.3.3.4.4 was explicitly BLOCKED pending a separate reviewed
immutable WebGPU generator input; it could not use this WGSL grammar fixture as a substitute.
