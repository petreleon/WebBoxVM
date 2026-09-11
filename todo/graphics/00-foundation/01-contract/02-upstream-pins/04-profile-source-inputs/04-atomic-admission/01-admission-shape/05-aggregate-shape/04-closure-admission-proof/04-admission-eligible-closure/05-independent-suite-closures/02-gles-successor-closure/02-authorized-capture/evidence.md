# F02.4.4.1.5.4.4.5.2.2 evidence

Revision: a2aa9776db85579d2b25e2455fba76a21b108fb4
Validation: child receipts, capture 6/6, binding 4/4, consumer 4/4, atomic reconciliation 4/4, `make test`, roadmap, and whitespace
Result: PASS
Artifacts: sealed six-member GLES marker and atomic blocked aggregate `6d7b8150b4fe71bdd7840a88ed9a4e35ce7192d024dec486821197fad83c5467`
Profile: raw GLES closure captured and replayable outside F02; atomic admission remains correctly blocked

Task ID and date: F02.4.4.1.5.4.4.5.2.2, 2026-09-11 Europe/Bucharest.

The immutable GLES root, four core selector members, and one explicit excluded extension were captured
and replayed offline with a marker published last. Binding and consumer checks prove the six identities
do not alias the active 17-input F02 inventory by ID or `(SHA-256, bytes)` and do not change F03.

The aggregate then validates the capture with the independent Docs and VCTS boundaries. It is
`atomically-blocked-unadmitted`: Docs still lacks a complete generated source authority/lineage closure,
and the 98-member VCTS default must-pass suite is broader than Vulkan 1.4 core, has 14 over-cap members,
and may not be locally filtered. Therefore this checklist is PASS for correctly guarded capture and
atomic rejection; it is not an admission or a graphics-compatibility result.

Focused totals were capture 6/6, binding 4/4, consumers 4/4, and aggregate 4/4. The full local gate
passed: 1,127 Rust / 0 failed / 3 ignored and 337 Node / 0 failed; roadmap and whitespace checks were
clean before completion markers. No CTS, guest application, browser renderer, conformance,
certification, or performance workload ran.

Decision and limits: the exact raw closure is available only as an external, unadmitted capture. It
cannot be used as evidence that GLES 3.2 works in WebBoxVM or that Docs/Vulkan suite inputs are ready.

Commit/push verification: `a2aa9776` is local on `codex/graphics-f01-baseline`; no remote push or CI
run occurred.

Next ready tasks: F02.4.4.1.5.4.4.2.2 establishes Docs closure, while F02.4.4.1.5.4.4.5.3 establishes
the VCTS core-manifest condition.
