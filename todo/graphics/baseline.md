# Starting evidence and existing commands

[Roadmap](README.md) · [Primary sources](sources.md)

The initial planning snapshot named `1354ed2` with uncommitted matrix/two-texture
work. F01 refreshed it on 2026-09-09 at clean `1026812` (`dd84070` is its bounded
v18 matrix/two-texture ancestor); that older snapshot is historical, not current.

Fresh F01 checks passed 133 focused Rust VirGL tests and 212 top-level browser-module
tests. Its full `make test` rerun passed 1,146 Rust tests (three ignored) and 336
browser-module tests. The retained Apple Metal WebGPU probe is still historical, not
rerun by F01. See the [F01 receipt](00-foundation/01-contract/01-baseline/evidence.md)
and [retained browser evidence](../../research/virgl-validation.md). This does not
complete any broader compatibility or performance task.

## Confirmed from current code

- [Capset implementation](../../emulator/src/devices/virtio_gpu/three_d/capset.rs)
  reports standard VirGL IDs 1/2 plus private ID 7; it does not expose Venus ID 4.
- [VirGL compatibility](../../research/virgl-compatibility.md) documents bounded
  resources, shader shapes and rendering state, not general Mesa/OpenGL support.
- [Venus foundations](../../research/venus-foundations.md) describes blob/shadow
  memory and private WBL1 preparation; it explicitly disclaims Venus interoperability.
- [Guest demo](../../guest/virgl-clear-demo/README.md) builds its own VirGL commands.
  [Native smoke](../../scripts/virgl_guest_transport_smoke.sh) is transport evidence;
  browser execution and standard Mesa guest programs require separate checks.
- [Worker polling](../../web/js/vm-worker/gpu-3d.js) emits one packet per eligible poll;
  [worker state](../../web/js/vm-worker/state.js) sets the GPU polling interval.
  [Display scheduling](../../web/js/gpu-display.js) and completion/readback are
  candidate bottlenecks to measure, not assumed performance wins.

The older source limiter covers configured source roots and selected root files;
it does not cover this new roadmap by itself. The roadmap checker covers this tree.
Existing research documents can already exceed 180 lines; split such files when
touching them. Preserve unrelated changes rather than reformatting them during planning.

## Commands that already exist

Run from the repository root. These are reference commands, not claimed run results.

```sh
git status --short
git diff --check
cargo test -p emulator --lib virgl --quiet
cargo test -p emulator --lib fence --quiet
cargo test -p emulator --test source_file_limits --quiet
node --test web/js/*.test.mjs
make test
make web-pkg
python3 scripts/check_graphics_roadmap.py
```

`node --test web/js/*.test.mjs` is the top-level browser-module subset. `make test`
recursively discovers browser tests and also runs Rust tests and the asset stamp check.
The F05 runner must fail zero-test selections rather than trusting exit code alone.

```sh
make -C guest/virgl-clear-demo
scripts/virgl_guest_transport_smoke.sh
make web-benchmark
```

The guest build needs the toolchain in its Makefile. The transport smoke boots an
installed guest disk and can be slow. The benchmark target verifies the configured
disk hash, builds Wasm and starts a server; it is not a graphics benchmark runner.
`make web-pkg` builds both serial and threaded Wasm and may need pinned build tools.
Record missing prerequisites explicitly. F05/I01 create the additional runners.

## Source boundaries

The implementation entry points are Rust VirtIO-GPU validation, guest memory and
completion; the VM worker transports work; browser modules own WebGPU resources.
The roadmap allows a tested ownership/coherence redesign while preserving observed
guest-memory semantics. A fast browser presentation or private packet route cannot
replace standard API correctness or same-GPU native performance evidence.
