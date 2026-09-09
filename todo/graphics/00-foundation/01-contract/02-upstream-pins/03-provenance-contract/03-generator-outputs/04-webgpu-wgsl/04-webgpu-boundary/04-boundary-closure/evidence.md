# F02.3.3.4.4.4 evidence

Revision: `babeb29974f101dff2aea428fe22172456f0e2ee` boundary closure; `4b2e852e0722aea40c51a7a9b13c8e5b66b699d2` collision regression
Validation: boundary 9/9; marker 7/7; renewal 3/3; provenance 10/10; inventory 6/6; source limits 6/6; local `make test`; roadmap; whitespace
Result: PASS
Artifacts: local boundary probe/tests; `webgpu-idl` SHA-256 `bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a`; marker SHA-256 `c56f2361017b8339f8a1a914898bc5ebb154c090c2ec1a13b0711b73d2afdb02`; raw lock `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`
Profile: local metadata-provenance input boundary only; no payload parsing, binding/API, browser, guest, renderer, VirGL/Venus, compatibility, profile, or performance behavior

Task ID and date: F02.3.3.4.4.4, 2026-09-09 Europe/Bucharest.

## Exact local boundary

The probe lock-validates the local F02.1 inventory and accepts only `webgpu-idl`. It
compares every frozen marker field: ID, source family, immutable URL, revision, source digest,
byte count, license, cache path, generated-code role, and provenance URL. The marker is loaded
under a collision-safe local module name, so a preloaded sibling `validate_fixture` module cannot
substitute a different task's identity.

The positive CLI result was:

```text
PASS: reviewed WebGPU WebIDL generator input is valid
```

## Hostile local cases

The hermetic suite rejects `webgpu-spec`, `wgsl-spec`, `wgsl-grammar-syntax`,
`webgpu-cts-buffer-map`, and `vk-gl-cts-api-version`. Renaming those records to
`webgpu-idl`, changing their family/role, or replacing a frozen field still fails. A synthetic
native-C header identity with the accepted name and role also fails. The suite rejects a stale lock,
missing/malformed inventory, and another identifier before lookup.

## Local validation record

From `/Users/petreleon/code/WebBoxVM`, these checks passed:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/webgpu_generator_boundary_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/03-webidl-generator-record/validate_fixture_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/03-generator-outputs/04-webgpu-wgsl/04-webgpu-boundary/02-lock-record-renewal/renewal_audit_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/01-provenance-record/provenance_record_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/01-input-inventory/validate_manifest.py --self-test
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

The focused suites passed 9/9, 7/7, 3/3, 10/10, and 6/6 respectively; source limits passed
6/6. `make test` exited 0 locally: Rust had 1,151 passed, 0 failed, 3 ignored; Node
had 337 passed, 0 failed, 0 skipped. macOS `xcrun` FSEvents/cache-path warnings were
environmental. The roadmap checker and whitespace check passed.

No upstream source payload was fetched, vendored, read, parsed, compiled, or executed. No remote CI
result is claimed. This receipt does not establish WebGPU bindings, browser WebGPU, guest-visible
VirGL/Venus, API compatibility, native comparison, or near-native graphics performance.
