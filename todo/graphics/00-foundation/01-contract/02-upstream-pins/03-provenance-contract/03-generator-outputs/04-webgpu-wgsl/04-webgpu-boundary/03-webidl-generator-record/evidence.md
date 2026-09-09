# F02.3.3.4.4.3 evidence

Revision: `418db2b82c1911ff545996d2e6723d5be58a30bd` WebIDL provenance marker
Validation: focused marker/renewal suites, direct validator, source limits, full local suite, roadmap, diff
Result: PASS
Artifacts: marker SHA-256 `c56f2361017b8339f8a1a914898bc5ebb154c090c2ec1a13b0711b73d2afdb02`; raw lock `08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`
Profile: metadata-only WebIDL input accounting; no parser, binding, API, guest, renderer, compatibility, or performance behavior

Task ID and date: F02.3.3.4.4.3, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: `418db2b82c1911ff545996d2e6723d5be58a30bd`; clean before the separate status/receipt diff.
Upstream manifest revision: schema-v2 raw `inventory.lock` SHA-256
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
Guest image and build hashes: not applicable; no guest was built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.

## Binding and scope

The generated JSON and schema-v2 sidecar accept exactly the reviewed `webgpu-idl`
record: WebIDL family, immutable URL/revision, source SHA-256, byte count, license,
cache/provenance paths, and explicit binding/interop-only role are frozen in both
generator and validator. The sidecar binds the current raw lock, exact command,
generator identity, artifact path, input triplet, and marker hash.

The marker says only that the WebIDL source is accounted for as a metadata input.
It contains no upstream WebIDL payload and neither reads a cache nor implements a
parser, binding generator, browser API, guest device, renderer, compatibility path,
or performance result. The legacy boundary probe remains untouched for
F02.3.3.4.4.4.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`, the direct validator passed and the
`validate_fixture_test.py` suite passed 7/7. It reproduces the marker offline,
rejects a stale lock, every frozen identity-field change, semantic/WGSL/CTS
substitutes plus a synthetic unknown native-C identifier, command/version/digest/license
drift, altered bytes, and scope changes even when their output hash is recomputed.
The renewal audit passed 3/3 and now
fingerprints all 12 current F02.3 lock-bound sidecars.

`cargo test -p emulator --test source_file_limits --quiet` passed 6/6.
`make test` exited 0 locally: Rust had 1,151 passed, 0 failed, 3 ignored, and
Node had 337 passed, 0 failed, 0 skipped. The macOS `xcrun` FSEvents/cache-path
warnings were environmental only. The roadmap checker passed 196 documents,
121 tasks, and 30 complete before this receipt; `git diff --check` passed.

The first focused development failure was the renewal audit seeing a temporary JSON
sidecar while marker tests ran in parallel. Test temporaries now live at the repository
root, outside that audit's provenance-contract discovery tree. A review also caught a
macOS-only temporary path; the repository-root location is portable and lock-safe.

No payload was fetched, vendored, parsed, compiled, or executed. No remote CI result
is claimed. This evidence does not establish WebGPU bindings, browser WebGPU,
guest-visible VirGL/Venus, API compatibility, a native comparison, or near-native
graphics performance.

Commit/push verification: feature commit `418db2b82c1911ff545996d2e6723d5be58a30bd`;
the receipt/status commit and remote SHA verification follow after its final checks.
Next ready task: F02.3.3.4.4.4 — Close the WebGPU generator-input boundary.
