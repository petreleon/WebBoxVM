# F02.3.3.1 evidence

Revision: `ccbe47aadd5284d668656591b54f3b5db84aa437` baseline; scoped feature diff pending commit
Validation: hermetic fixture-generator, provenance, source-limit, full-suite, and roadmap checks
Result: PASS
Artifacts: one four-line local provenance marker plus its JSON sidecar; no upstream protocol bytes
Profile: fixture-only generator contract; no Venus guest, runtime, API, rendering, or network behavior

## Bound record

The sidecar binds F02.1 raw inventory-lock SHA-256
`cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b` solely to
`venus-protocol-registry` / `d92839bc728fa9ad9a7decdc6b91df6fa1a0fb26cffae4009865f18a789e0535`
under its recorded `Apache-2.0 OR MIT (SPDX file notice)` license. The marker byte hash is
`4a87aaf839dce537622ca76fedbf97df1fc77cc5b1840b924772a1f10bb2fde0`.

`venus_fixture_generator.py` accepts only that identity and emits the marker without opening,
copying, or parsing the upstream registry. `venus_record.py` requires the exact command, approved
generator identity/version, one input, record-relative artifact path, and current artifact hash.

## Commands and actual results

```sh
cd todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/01-venus-codec
PYTHONDONTWRITEBYTECODE=1 python3 venus_record_test.py
PYTHONDONTWRITEBYTECODE=1 python3 ../../01-provenance-record/provenance_record_test.py
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

The Venus suite passed 6/6 and the generic record-contract suite 9/9. Negative cases reject an
incorrect generator version/command, stale or unknown input identity, WebGPU reference and Mesa
runtime substitutions, a stale output digest, and modified fixture bytes. Source limits passed 6/6.
Full-suite and final structural outputs are recorded after the shared parent markers are reconciled.

This is not a generated Venus codec, does not prove compatibility with Mesa or a guest application,
and does not execute any graphics workload. Next ready work is the other F02.3.3 family children;
the WebGPU/WGSL boundary remains open because its current immutable inputs are reference-only.
