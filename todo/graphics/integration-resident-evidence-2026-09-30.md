# Automated state inventories and resident-owner integration

Revision: `5e9561d3975536f9e5090616a9bf0fdc89d5f70d`
Validation: full local suite, fresh serial/threaded Wasm, real WebGPU and Linux guest
Result: PASS for these bounded checks; full graphics and R02 acceptance remain open
Artifacts: local logs and observations under `.artifacts/graphics/2026-09-30-validation/`
Profile: raw source vocabulary and resident-resource ordering/lifetime

Date: 2026-09-30 Europe/Bucharest. Starting revision:
`920328321f5c3fafee1bade3d823b1c26458abcb`, initially clean.
Implementation commits, tested together with identical code bytes before committing:

- Inventory automation: `699a59fddf076890f29178b4016779174735e02b`.
- Runtime owner retention: `2bef6ee62d764da58bb3bdda8cfcf5ad0abca7f3`.
- Browser release ordering: `5e9561d3975536f9e5090616a9bf0fdc89d5f70d`.

The 199-file code manifest hashes compact sorted JSON mapping changed/new code
paths to their file SHA-256 values, including Makefile targets and browser asset
versions. Manifest digest: `4e60ca4b7e08502aed4cf0d5e973baab008367c613700acdb2ab2d9a2486ebd8`.
`retained-owner-code-manifest.json` preserves the mapping. Documentation status
updates follow in a separate commit; they do not change the tested implementation.

## Full local checks

- `/usr/bin/time -p make -j4 test`: exit 0, 903.92 seconds; 498 Python tests
  across 81 suites, 1,173 Rust tests passed with 3 existing ignored cases, and
  342 web tests passed. No failures; no additional tests were skipped.
- The Rust run includes all six source-file-limit tests; maintained files remain
  at most 180 physical lines. Final documentation gets a separate limit check.
- `make graphics-inventory-automation-test`: 34 passed, exit 0.
- `make web-pkg`: exit 0; fresh serial/threaded wasm64 release builds and bindings.
- `git diff --check` and the roadmap checker passed. This receipt closes only
  three raw declaration leaves; the full foundation and state parent remain open.

Post-documentation checks passed: six source-limit tests and roadmap validation
of 559 documents / 330 tasks / 119 PASS-complete / 81 superseded. All 199 tested
code files still match their manifest. `final-checks.json` records the 33 remaining
active foundation leaf IDs. The first final roadmap check rejected qualified
`Result` headers on the three closed raw leaves; their headers were corrected to
the standard exact `Result: PASS`, preserving scoped acceptance and evidence.

| Local artifact | SHA-256 |
| --- | --- |
| `retained-owner-make-test.log` | `9a72a67450adfb079bf629b3219ceaa0f569f876ac13abc4f6159eac1cb848cf` |
| `retained-owner-web-pkg.log` | `08324e787bdff5f6df9e82edb52ef82dc05df760f1009786d2ef5954864e6d26` |
| `retained-owner-webgpu.json` | `dad16ef65980b530ebc30dd493c716ffb4cebd9d2b75d98a2ea17835b1e70265` |
| `retained-owner-code-manifest.json` | `33d8913314d7d108bd66161f6dbc7fa34280fd41274a1ec9bd01b93ea466fbd8` |

## Finite source automation

The fixed `scripts/graphics_inventory_batch.py` entrypoint retains the five-task
vertex default and adds exactly the four existing state/execution routes.
State check/regeneration targets and all focused tests are wired into Makefile.
The [partial source aggregate receipt](00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/04-state-execution-command-slices/evidence.md)
records 97 facts: 12 partial lifecycle rules, 55 draw/raster/compute forms,
six pixel commands and 24 debug/special/context-query forms.

Measured state check/regeneration before the final guard correction took 8.422 /
7.586 seconds. Final regeneration with the corrected guard passed during the full
suite in 11.261 seconds; all 20 files were byte-identical to stored artifacts.
Final vertex regeneration preserved all five artifact files and 36 raw forms.
`final-inventories/summary.json` records final commands, hashes and comparisons.
The state report still has `complete: false`, only lifecycle incomplete, no missing
selected tasks. Three complete raw declaration leaves are closed, with no new task
IDs. Active foundation leaves fall from 36 to 33: 32 F03 leaves and one F04 leaf.

Independent review reproduced a snapshot defect: absolute `__pycache__` path
exclusion hid normative-cache or authority changes. The first regression run
exited 1: seven tests, five failing subcases. Exclusion now applies only to code
bytecode beneath relative Python-cache directories; external cache files and
non-bytecode authority files always remain fingerprinted. All seven snapshot
tests pass inside the 34-test automation suite. First output is retained in
`snapshot-path-before.log`, SHA-256
`e4cb7c4dad12a00787ae74c5898e1e44609435f876bbe436d39321f0b1cdae0e`.

The first final artifact comparator exited 1 after successful vertex generation:
its filename filter omitted the existing `*_inventory.json` convention. The
corrected comparator checks exact basenames against all contract JSON and reuses
the successful generated bytes. `final-inventories/first-comparison-failure.json`
preserves that harness failure; no source or artifact was changed to conceal it.

## Runtime and browser behavior

Unref, context destruction and synchronous CPU writes now preflight resident
release capacity before mutation. Accepted resident readbacks retain the exact
allocation owner and its byte/count charges until completion or cancellation;
late completion preserves a newer resource with the same ID. Overlapping
readback/write/copy/sample work is rejected in both admission orders. A final
release remains lossless even with the existing 16-slot queues saturated.
The 12 new regressions verify response codes, preserved bytes, exact accounting,
releases and cancellation. Focused GPU tests passed 189/189; all limits are unchanged.

Runtime fingerprint: `dbb968e715a271d9554a78b981843632e2627ea848a8efecce3f92c9cd654bda`.
It hashes each of the 22 sorted runtime paths, NUL, file bytes, NUL.
`.artifacts/graphics/r02-runtime-guest/retained-owner-source-validation.json`
records each file hash, reproduced pre-fix failures and remaining obligations;
receipt SHA-256 `4897c910000b155c23c870e36ff02b19e0aeee42f989d53870c4fcac77b6f1bd`.

Browser release controls now queue behind accepted producer publication and
readback, without draw diagnostics or acknowledgments. Epoch checks cancel stale
controls across reset/device loss, including a second check after helper await.
The first pre-fix three-test run had one pass/two failures, exit 1, exposing early
publication release and readback destruction. Four final regressions pass in the
complete web suite; the broader targeted browser group passed 20/20.

Fresh Wasm was served with `scripts/serve_web.py` and the page reloaded after
build. Reproduce the real probe with the Playwright skill wrapper:
`playwright-cli --session graphics-lifetime-20260930 run-code --filename
scripts/check_virgl_resident_lifetimes.mjs` on localhost:8765.
An Apple/Metal-3 non-fallback adapter passed 32 partial-readback/release cycles
(1,024 checked pixels, BGRA [191,128,64,255], tolerance 1), 32 publication/release
cycles, reset sequence reuse and actual device destruction/reacquisition.
Tracked resident textures: 66 created, 66 explicitly destroyed, zero live;
two devices, generations 1 and 2, no GPU errors. This counts explicit resident
texture lifetime, not physical driver memory. In-flight loss cancellation is
covered by the deferred unit test; the real loss probe occurs after publication.
The owned browser session and localhost server were closed after verification.

## Real guest and remaining acceptance

`cargo run -p emulator --release --example virgl_guest_transport_smoke --
output/webboxvm-final-install-compact.wbdisk guest/virgl-clear-demo/build/virgl-clear-demo`
passed with unchanged 900-second / 20-billion-step / 2-million-step-chunk defaults.
Real Linux produced both module/demo markers and all 19 unique completed
sequences: 281.251 seconds, 5,328,140,000 steps. Disk SHA-256:
`97d819803774d67c9aabaa19f336f066656cc5235b5e8276cb8dc14fdff6217d`.
The retained-owner transport log SHA-256 is
`d47295a594e6673b1acf0c9a771f64ab8e7520f5f897d6e98a1ad3e5e160cb47`;
`retained-owner-result.json` records defaults, hashes, command and counts.
Earlier guest receipts remain unchanged. This uses handcrafted standard capset-1
packets and CPU reference completion; it does not establish stock Mesa behavior.

R02 still lacks unified VM/device handles, all foreign-context cases and general
resident-texture CPU replay. CTS, full API profiles and near-native performance
remain unverified. The next independent ready real-guest action is I01: its
[exact Mesa input capture](05-guest-validation/01-real-clients/01-guest-image/input-capture-evidence.md)
is verified, but no stock-Mesa image has been built. No Actions workflow is
configured for these changed paths; these are local checks, separate from CI.
