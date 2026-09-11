# F02.4.4.1.5.4.4.5.2 evidence

Revision: a2aa9776db85579d2b25e2455fba76a21b108fb4
Validation: integration 3/3; closure contract 3/3; capture 6/6; binding 4/4; consumers 4/4; atomic aggregate 4/4; `make test`; roadmap; whitespace
Result: PASS
Artifacts: sealed GLES closure `fcca79aa08ca156cddca92b624e66513526109219b84b253bd513879f34419c4` and aggregate `6d7b8150b4fe71bdd7840a88ed9a4e35ce7192d024dec486821197fad83c5467`
Profile: immutable GLES selector closure captured but not admitted into active F02/F03

Task ID and date: F02.4.4.1.5.4.4.5.2, 2026-09-11 Europe/Bucharest.

The previous rejected GLES multi-file candidate now has a complete six-member immutable raw closure:
one root, four ordered core members, and an explicit extension exclusion. Its cache marker is replayable
offline, its integration remains a separate `gles-cts` wrapper, and no local reconstruction or
cross-wrapper substitution is accepted.

The closure is still not active or admitted. Its atomic aggregate retains all F02/F03 effects false and
records Docs/VCTS as independent mandatory blockers. PASS therefore means the bounded closure and
fail-closed rejection mechanism are complete; it does not mean GLES source admission, GLES CTS, guest
OpenGL/GLES support, browser behavior, conformance, certification, or performance.

The listed focused suites passed 3/3, 3/3, 6/6, 4/4, 4/4, and 4/4. `make test` passed 1,127 Rust tests
with 0 failed and 3 ignored plus 337 Node tests with 0 failed. The roadmap and whitespace checks were
clean before completion markers.

Commit/push verification: `a2aa9776` is local on `codex/graphics-f01-baseline`; no remote push or CI
run occurred.

Next ready tasks: F02.4.4.1.5.4.4.2.2 establishes Docs closure, while F02.4.4.1.5.4.4.5.3 establishes
the independent VCTS core-manifest condition.
