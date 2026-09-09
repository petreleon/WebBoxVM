# F02.3.3.2 evidence

Revision: `ccbe47aadd5284d668656591b54f3b5db84aa437` baseline plus scoped uncommitted leaf files
Validation: offline fixture validator, 5-case target suite, contract/inventory suites, source limit, whitespace
Result: PASS
Artifacts: fixture and sidecar below; no upstream payload bytes
Profile: fixture-only provenance binding; no GL/GLES API, runtime, guest, or browser behavior

## Bound identity

The sole generated-input reference is `opengl-gles-registry` from the F02.1 manifest:

- Inventory-lock SHA-256: `cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b`.
- Input SHA-256: `b9ca2cfa5c676e901c20d34af3407f1687cde0f1336a5ff7a8974d04c7494ad3`.
- License: `Apache-2.0 (SPDX file notice)`.
- Generator: `f02-gl-gles-fixture` version `1`.
- Output: [fixture-output.json](fixture-output.json), SHA-256
  `bdcec164579481b459eee7d2ab54bc867c5f010dd5bc28e8a94635b707397d91`.
- Sidecar: [fixture-output.provenance.json](fixture-output.provenance.json), SHA-256
  `6bfc2af39f9923008c612aeed501cc5cf0aee80a157a86b655ec25d8b24cc5fc`.

The recorded command is in the sidecar and regenerates only a small identity marker. It reads the
reviewed inventory metadata, not `gl.xml`; the tracked fixture contains no registry, GLSL, or ESSL
source bytes.

## Commands and results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/02-gl-gles-registry/validate_fixture.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/02-gl-gles-registry/validate_fixture_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/01-provenance-record/provenance_record_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
```

The direct validator passed. The target suite passed 6/6 tests; it regenerates the fixture offline,
rejects both semantic specifications, a wrong generator version, a stale input digest, a wrong
sidecar output hash, and a changed fixture byte. The shared contract and inventory suites passed
9/9 and 4/4 respectively; source limits passed 6/6; `git diff --check` passed.

The first development failure was an incorrect parent-directory lookup for `provenance_record.py`.
It was corrected before the recorded validation. The target test keeps each malformed sidecar in a
separate temporary file, so changing a fixture byte is checked against the original valid sidecar.

Global `make test` and the roadmap check are owned by the shared parent reconciliation because
other F02.3 children were completing concurrently. No file was staged, committed, or pushed.

Next ready work: the remaining F02.3 generator/provenance children and final parent validation.
