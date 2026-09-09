# F01 evidence

Revision: `10268125dc84d9fd451c8952726420ab6fd7b815` (clean starting tree)
Validation: focused Rust and Node suites; full `make test`; roadmap/source-limit checks
Result: PASS
Artifacts: four local guest/disk inputs below, SHA-256 recorded
Profile: planning-only baseline; no GL/GLES/Vulkan profile has been frozen (F03)

## Snapshot

Task ID and date: F01, 2026-09-09.

Tested commit and dirty diff hash: `10268125dc84d9fd451c8952726420ab6fd7b815`,
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` before
this receipt; `git status --short` was empty. The historical `1354ed2`/dirty snapshot
in `todo/graphics/baseline.md` was reconciled to this one.

Capsets and limits observed from `emulator/src/devices/virtio_gpu/three_d/capset.rs`:
VirGL `1`/v1/308 bytes, VirGL2 `2`/v2/1,376 bytes, and private WBG3 `7`/v1/32 bytes.
There is no Venus capset `4`; VirGL2's trailing fields remain zero. The actual slice is
bounded: 64 KiB VirGL submits, two fragment samplers, 4 KiB/256-token shaders, three
vertex buffers, 1,023 input vertices, 16 pending submits, and 2 MiB pending bytes.
It does not establish Mesa, OpenGL/GLES contexts, Venus, or Vulkan interoperability.

Upstream manifest revision: none; F02 owns immutable upstream protocol/test pins.

Guest image and build hashes:

- `.artifacts/Image.gz`: 41,181,696 bytes,
  `71a7e2ada3668a3e76982dc89225bfc0da674ed7f6a37a8ce2191cbba3957ae6`.
- `.artifacts/debian-arm64-netinst.iso`: 735,358,976 bytes,
  `3f8211e759d19370d50e1d853859b66ecba62700d712214a8a65ed26c6d08ecc`.
- `output/webboxvm-final-install-compact.wbdisk`: 1,259,034,724 bytes,
  `97d819803774d67c9aabaa19f336f066656cc5235b5e8276cb8dc14fdff6217d` (Makefile pin).
- `output/webboxvm-final-install.wbdisk`: 1,535,564,860 bytes,
  `68fff3b246aa6bdd207891bb7479ef8aa803ea477da0d6be64db9b516c77dc40`.

## Commands and observed results

Working directory: `/Users/petreleon/code/WebBoxVM` on macOS 26.6.2 (25G83).
Toolchains: git 2.52.0, GNU Make 3.81, rustc/cargo 1.93.0, Node 26.0.0,
npm 11.12.1, Python 3.14.6, and wasm-bindgen 0.2.122. The stable Apple target and
`wasm32-unknown-unknown` are installed; the repository's custom threaded wasm-bindgen
exists under `.artifacts/tools/wasm-bindgen-memory64-threads/bin/wasm-bindgen`.

Expected result: every selected suite exits zero with a nonzero passing case count.

```sh
git status --short
cargo test -p emulator --lib virgl --quiet
node --test web/js/*.test.mjs
make test
cargo test -p emulator --test source_file_limits --quiet
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Actual results: the focused Rust command passed 133, failed 0, ignored 0, filtered 997;
the top-level Node command passed 212, failed/cancelled/skipped/todo 0. `make test`
passed 1,146 Rust cases, failed 0, ignored 3, and passed 336 recursively discovered
Node cases with 0 failures/cancellations/skips/todos. The source-file limit passed 1/1.
Before the status edit, the checker reported 127 documents, 86 tasks, 0 complete and
`Ready: F01`; the final checker result is recorded by the completion validation.

Matrix v18 coverage was included in both lanes: Rust's
`matrix_two_textures_keep_raw_uvs_for_webgpu` and the Node matrix packet, display, and
WebGPU renderer tests verify two independent bounded sampler/texture snapshots and
malformed-frame rejection. No required suite had a failing subcheck.

## Boundaries and retained evidence

Raw logs/images: no large transient test log is checked in; rerun the exact commands
above against the recorded revision. The four hashes above identify available inputs.
The prior Apple Metal v18 browser pixel probe remains a retained historical result in
`research/virgl-validation.md`, not an F01 rerun.

`playwright-cli`, Chromium/Chrome, `glxinfo`, `vulkaninfo`, and `virgl_test_server`
were unavailable on PATH. QEMU and the guest compiler were present, but no stock Mesa
guest application, native reference run, real-browser adapter probe, conformance suite,
or same-GPU performance comparison was run. Node tests use controlled test doubles, so
they do not prove a hardware WebGPU route or rule out browser fallback.

Performance conditions: P01 has not frozen the same-hardware protocol; no throughput,
frame-time, latency, or near-native claim is made. First failing subcheck/blocker: none
for F01; the unavailable real-guest/browser/reference lanes remain blockers for later
tasks, not failures hidden as passes.

Decision and limits: this receipt completes only the reproducible starting-state record.
It deliberately preserves the narrow bounded VirGL implementation and does not advertise
OpenGL 4.6, GLES 3.2, Vulkan 1.4, Mesa compatibility, or near-native performance.

Commit/push verification: the F01 documentation commit and remote SHA are verified in
the task handoff after the post-edit checks; no remote CI result is claimed locally.
Next ready task: run the roadmap checker after this receipt; expected independent leaves
are F02 (upstream pins) and F06 (feasibility), subject to its dependency result.
