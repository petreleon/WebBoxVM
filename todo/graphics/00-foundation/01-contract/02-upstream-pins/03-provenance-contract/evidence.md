# F02.3 evidence

Revision: `b50dcbedd79e719ace50628daa773b8eac3cb81a` closure implementation commit
Validation: child receipts plus fresh 17-input cache closure, focused suites, source limits, local `make test`, roadmap, and whitespace
Result: PASS
Artifacts: six ABI and six generator provenance records closed against the immutable F02.1 inventory
Profile: provenance metadata and source-byte verification only; no guest, renderer, browser, compatibility, or performance claim

Task ID and date: F02.3, 2026-09-09 Europe/Bucharest.

F02.3 is complete because its four checked children now have receipts:

- [record contract](01-provenance-record/evidence.md)
- [ABI bindings](02-abi-fixtures/evidence.md)
- [generator bindings](03-generator-outputs/evidence.md)
- [fresh provenance closure](04-provenance-validation/evidence.md)

The final child independently fetched all 17 immutable inputs into a new external cache, reused and
re-hashed all 17 offline, and verified the complete cache against all 12 reviewed F02.3 sidecars. The
same closure fails if a cache input is unavailable. The lock identity is
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.

The completed local gates are recorded in the child receipt: the closure suite 7/7, source limits 6/6,
the inherited focused suites, `make test`, roadmap verification, and whitespace verification all passed.
No payload was committed; the 18,953,659-byte cache remains external.

F02.3 makes a fail-closed provenance statement only. It does not establish a guest ABI/device, VirGL,
OpenGL/GLES, Venus/Vulkan, WebGPU runtime, browser GPU route, renderer, compatibility profile,
native-comparison result, remote CI result, or near-native performance.
