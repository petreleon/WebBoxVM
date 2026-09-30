# R02 allocation-retention progress

Revision: `589577cbcd73706e708be36e1f00d16236619938`
Validation: focused Rust behavior regressions and existing VirGL/context/source-limit checks
Result: PASS for the focused checks; R02 remains incomplete
Artifacts: runtime/regression sources and linked bounded guest/browser observations
Profile: protocol-independent allocation lifetime; no additional advertised API support

Date: 2026-09-30 Europe/Bucharest.
Runtime content SHA-256: `24df63b038e918b63a620f8c558a2e4a1aa730892f40888fb748ea137e9fd5eb`.
The content digest includes each changed/new emulator path, NUL, file bytes and NUL, in sorted
`git status --porcelain --untracked-files=all` order. It excludes documentation and other workers.

## Reproduction and invariant

Before the fix, `cargo test -p emulator --lib
unref_recreate_before_late_clear_preserves_new_allocation --quiet` failed with exit 101, 0/1:
the old queued clear replaced the sentinel bytes of the newly created resource with the same ID.

Each accepted asynchronous job now pins allocation identity `(resource ID, generation)`.
Unref removes guest visibility and retains the exact allocation, byte charge and object-count charge
until its final dependent job completes or is cancelled. Completion resolves retained allocation
storage synchronously, restores any newer live allocation, and never damages its scanout or
promotes the retired target as a resident object. Context-generation rejection remains enforced.
Reset discards pending jobs and retained allocations together; old completions remain invalid.

Reference lookup is O(1); retain/reclaim scans are bounded by the existing 16 pending-job limit.
Completion moves resource ownership rather than copying its pixels. Allocation generations fail
closed at counter exhaustion instead of wrapping. Existing resident-copy/sample deletion guards
remain; broadening those lifetime semantics requires separate implementation and verification.

Review reproduced release saturation: `cargo test -p emulator --lib
saturated_retired_resident_completions --quiet` first failed with exit 101, 0/1, because only
16 old producers were released while 16 completed replacement producers leaked. Completed resident
jobs now reuse their existing pending slot for a release transport record when the 16-entry release
deque is full. No limit is raised. The guest acknowledgment is accepted once, and new submissions
backpressure until polling removes these release records and their 12-byte packet charge.

## Focused results

- `cargo test -p emulator --lib resource_lifetime --quiet`: 9 passed, 0 failed/ignored.
- `cargo test -p emulator --lib saturated_retired_resident_completions --quiet`: 1 passed.
- `cargo test -p emulator --lib devices::virtio_gpu::tests::lifecycle --quiet`: 13 passed.
- `cargo test -p emulator --lib virgl --quiet`: 133 passed, 0 failed/ignored.
- `cargo test -p emulator --lib devices::virtio_gpu::tests::context --quiet`: 4 passed, 0 failed/ignored.
- `cargo test -p emulator --test source_file_limits --quiet`: 6 passed, 0 failed/ignored.
- `git diff --check`: exit 0.

The regressions cover clear/unref/recreate/late completion, two retired generations of one ID,
last-job retention, completion ordering, failure, cancellation, reset, context destruction,
resident release packets, a real 128 MiB resource-budget boundary, and GPU readback into an old
1024x768 allocation while the new allocation is 8x8. They assert guest response codes, preserved
new bytes, exact byte/count accounting and absence of new scanout damage/residency.
The saturation regression delivers all 32 distinct old/new releases, verifies the unchanged
16 pending-slot limit and 192-byte release charge, rejects new submissions while saturated, and
accepts new work after draining. Small internal render targets are used without changing KMS scope.

The first source-limit run found a 181-line test aggregator. The new tests were then organized
beneath the existing lifecycle module, and all six source-limit tests passed.

## Remaining parent acceptance

[Complete local integration](../../../integration-evidence-2026-09-30.md) passed: `make test`,
fresh serial/threaded `make web-pkg`, the existing browser renderer probe and standard real-guest
transport lane. These checks do not prove all R02 identity obligations, VM/device generations,
cross-context rejection, browser object retention, device loss, conformance or performance.
The GPU readback regression covers `VirglBatch` output, not `VirglResidentReadback`: preserving
resident owner metadata across unref remains unfinished. General `forget_resident` release-capacity
handling also remains an R02 obligation; the lossless correction here covers completed jobs.
