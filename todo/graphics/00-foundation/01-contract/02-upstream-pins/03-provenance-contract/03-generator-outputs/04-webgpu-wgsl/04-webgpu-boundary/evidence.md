# F02.3.3.4.4 evidence

Revision: `babeb29974f101dff2aea428fe22172456f0e2ee` boundary closure; `4b2e852e0722aea40c51a7a9b13c8e5b66b699d2` collision regression
Validation: boundary 9/9; marker 7/7; renewal 3/3; provenance 10/10; inventory 6/6; source limits 6/6; local `make test`; roadmap; whitespace
Result: PASS
Artifacts: `webgpu_generator_boundary.py`, its hermetic tests, reviewed `webgpu-idl`, marker SHA-256 `c56f2361017b8339f8a1a914898bc5ebb154c090c2ec1a13b0711b73d2afdb02`, raw lock `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`
Profile: immutable local metadata-provenance eligibility only; no payload parsing, generator implementation, WebGPU API, guest, browser, renderer, compatibility, or performance behavior

Task ID and date: F02.3.3.4.4, 2026-09-09 Europe/Bucharest.

The closed parent accepts exactly the frozen `webgpu-idl` WebIDL binding/interop record,
including its immutable URL/revision, source SHA-256
`bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a`, byte count, license,
cache/provenance paths, and explicit role. It rejects every other identifier and any differing field.
The local F02.1 layout validates its canonical raw lock before the identity comparison.

The boundary's 9/9 suite includes the positive command-line result, all frozen-field mutations,
semantic/WGSL/grammar/CTS substitutes, renamed role/family spoofs, a native-C spoof, malformed or
missing inventory, stale lock, and a real sibling-module preload collision. The marker suite passed
7/7; its provenance sidecar and output remain metadata-only. The renewal audit passed 3/3 over the
12 current F02.3 lock-bound sidecars; generic provenance passed 10/10; inventory self-test and
source limits each passed 6/6. `make test`, the roadmap checker, and `git diff --check`
passed locally.

The probe loads only checked-in metadata and does not fetch or read upstream WebIDL bytes. No browser
or GPU execution occurred. No remote CI result, WebGPU binding/API, guest-visible graphics,
VirGL/Venus translation, profile conformance, native comparison, or performance result is claimed.
