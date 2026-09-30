# Inventory automation and GPU lifetime integration

Revision: `589577cbcd73706e708be36e1f00d16236619938`
Validation: source/unit integration, fresh Wasm builds, existing browser and guest lanes
Result: PASS for these bounded integration checks
Artifacts: local logs/observations under `.artifacts/graphics/`; hashes below
Profile: source vocabulary and allocation retention; no full API/performance claim

Date: 2026-09-30 Europe/Bucharest.
Inventory implementation: `96f2d56a7924e723af2e76c73dff60719225af6b`.
Runtime implementation: `589577cbcd73706e708be36e1f00d16236619938`.
Checks ran against the identical source bytes before these two commits. The following
documentation commit records results and closes only the bounded source groups.

## Complete local checks

- `make -j4 test`: exit 0; 455 Python tests across 77 suites, 1,161 Rust tests
  passed (3 intentionally ignored), and 338 web tests passed. No failures.
- `cargo test -p emulator --test source_file_limits --quiet`: 6 passed, exit 0.
- `make web-pkg`: exit 0; fresh serial/threaded wasm64 release builds and bindings.
- `git diff --check`: exit 0.
- Roadmap checker: 550 documents, 330 tasks, 116 PASS-complete, 81 superseded before
  adding this integration receipt; dependencies, links and line limits passed.

Logs in `.artifacts/graphics/2026-09-30-validation`:

| Artifact | SHA-256 |
| --- | --- |
| `make-test-after-cache-restore.log` | `9bdab72863b2b90a48ce8a761d20650996169276f1fd17fbf2450ab625850416` |
| `web-pkg.log` | `7089d7972067150a5c34f7945c6020ffdc411bc43757f6b016db8111b835553d` |
| `webgpu-observations.json` | `c8e981737f255fe67cb520b173af7cbc253f80293741baf45f8eb45ce2765358` |

The first `make -j4 test` stopped with exit 2 at
`graphics-opengl-command-object-raw-inventory-test` (child exit 1): the external
pinned OpenGL PDF cache file was absent. Its official immutable bytes were restored
with the existing normative source-cache tool, without changing source or limits:
3,003,752 bytes, 851 pages, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`.
The affected focused suite then passed 5/5, followed by the complete successful run.
The original partial run is preserved as `make-test.log`, SHA-256
`a8627ec7533958f69dde45ac685acee1b55b14ee459a9ce897b92c2c0a056318`.

## Browser and real guest

Fresh `web/pkg` was served through `scripts/serve_web.py` on localhost:8765.
`playwright-cli --session graphics-20260930 run-code --filename
scripts/check_virgl_matrix_texture_multiply.mjs` passed on an Apple/Metal-3 WebGPU
adapter: sampler words 4242, 4224 and 12946 all produced inside BGRA
`[50,40,200,255]`, outside `[77,51,26,255]`, and successful acknowledgments 1–3.
GPU error scopes were empty. The page console recorded only a favicon 404.
This existing renderer probe does not exercise the allocation-reuse regression.

`make -C guest/virgl-clear-demo` built a 52,984-byte static AArch64 ELF. Then:

```sh
cargo run -p emulator --release --example virgl_guest_transport_smoke -- \
  output/webboxvm-final-install-compact.wbdisk guest/virgl-clear-demo/build/virgl-clear-demo
```

The unchanged default gate (900 seconds, 20 billion steps, 2 million steps/chunk)
passed with `VIRGL_SMOKE_MODULE_OK`, `VIRGL_TEXTURE_DEMO_PASS` and all 19 unique
completed sequences. It reached a real Linux shell and exercised standard capset-1
clear/draw/upload/copy/readback/texture/depth/batch paths in 281.198 seconds,
5,328,140,000 steps. Disk SHA-256 matches the Makefile's configured identity:
`97d819803774d67c9aabaa19f336f066656cc5235b5e8276cb8dc14fdff6217d`.
The transport log is `.artifacts/graphics/r02-runtime-guest/standard-transport.log`,
SHA-256 `55f4bab1c1a2cd67b385f6f00d502bf8375e98bd4e1d8bf43136e702cce77235`.
`standard-result.json` records the command, limits, disk/demo/harness hashes and counts.
The initial 180-second exploratory probe timed out during shell readiness (exit 1);
its `result.json` and `transport.log` remain intact. No GPU command had run then.

The guest lane uses handcrafted VirGL packets and CPU reference completion. It does
not establish stock Mesa, browser allocation-retention loops, CTS or performance.
R02, the full foundation and the overall graphics goal remain open. No GitHub Actions
workflow is configured for these changed paths; this receipt reports local checks.
