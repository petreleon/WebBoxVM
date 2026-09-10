# F02.4.4.1.5.2.3.4 aggregate receipt — staged Docs build witnesses

Revision: `b2e47d3f` completed final child implementation
Validation: four child receipts, 4 focused marker tests, source limits, roadmap checker, and `make test`
Result: PASS
Artifacts: transient private external caches only; all final verification roots were removed
Profile: Vulkan 1.4 recorded Docs witnesses only; unadmitted and not cutover-ready

## Aggregate result

The four children now form one bounded staging chain without changing an active F02 consumer.
The contract child fixed the live plan, private external-root boundary, state grammar, and
no-overwrite descriptor primitives. The input child retained the canonical 1,760-member manifest
(298 raw and 1,462 derived) plus its receipt. The output child separately retained two complete
2,530-file, 17,019,466-byte trees under their recorded run identities; equal content digests never
collapse the two origins. The final child rehashes every retained payload and publishes the sole
self-hashed marker only after the exact combined closure is present.

The validated plan digest is
`5dcc55e3cd979010d73142e7dde13361e0b36f64ff42aaa1e436a8a5eab5de77`. Its full staged cache
contains exactly 6,823 regular files after marker publication: 1,761 input files, 5,061 output
files, and one marker. The marker remains `staging-only-unadmitted`, with both admission booleans
false, and its grammar binds the same reviewed scope/comparison/build-witness context as the
preceding receipts.

## Verification and limits

The final focused marker suite passed 4/4 in 733.984 seconds; the source-file limit suite passed
6/6; and `make test` exited 0 with 1,127 emulator unit tests passed (3 ignored), boundary and
graphics-roadmap checks passed, and Node 337/337. `git diff --check` passed. `make web-pkg` was
not applicable because this aggregate changes no Rust, Wasm, or browser code. Remote CI was not
run.

This completes only captured-build staging. It does not replace a fresh official Docs build, prove
the actual Docs closure, admit any F02 source, activate a cache, or make a graphics/conformance or
performance claim. The known same-UID post-final-check mutation limitation remains bounded by
rehashing on every publish/reuse.

The next child of the Docs binding parent is `F02.4.4.1.5.2.3.5` — prove the actual Docs closure.
