# F02.4.4.1.5.4.4.5.2.2.3 evidence

Revision: a2aa9776db85579d2b25e2455fba76a21b108fb4
Validation: child receipts plus integration 3/3, capture 6/6, binding 4/4, consumer 4/4, atomic reconciliation 4/4, `make test`, roadmap, and whitespace
Result: PASS
Artifacts: atomic blocked aggregate `6d7b8150b4fe71bdd7840a88ed9a4e35ce7192d024dec486821197fad83c5467`
Profile: sealed GLES capture and unchanged F02/F03 consumers; independent Docs/VCTS boundaries remain unadmitted

Task ID and date: F02.4.4.1.5.4.4.5.2.2.3, 2026-09-11 Europe/Bucharest.

All three children are now complete: the sealed capture binding, unchanged consumer validation, and
the atomic aggregate. The aggregate binds the same six-member GLES closure without copying it into
active F02, marks all F02/F03/support/conformance/performance effects false, and preserves all six
ordered F03 missing input IDs.

The historical blocked receipt is context only. The current aggregate moves no state forward: it keeps
Docs unadmitted until a complete generated-source authority/lineage closure exists, and keeps VCTS
unadmitted until Khronos publishes an immutable explicit Vulkan 1.4 core manifest rather than a
default must-pass suite or local filter. Its two current independent global blockers are recorded in
the child receipt.

Validation is recorded in the three child receipts. The final local gate for this feature revision was
`make test` with 1,127 Rust passed / 0 failed / 3 ignored and 337 Node passed / 0 failed; the roadmap
checker and `git diff --check` exited zero before completion markers. No real CTS, guest API, browser,
conformance, certification, or performance result was run.

Decision and limits: PASS is a correct blocked-state reconciliation, not graphics compatibility. It
does not admit GLES, Docs, or Vulkan CTS sources and cannot establish any guest-visible feature.

Commit/push verification: implementation revision `a2aa9776` is local on
`codex/graphics-f01-baseline`; no remote push or CI run occurred.

Next ready tasks: F02.4.4.1.5.4.4.2.2 and F02.4.4.1.5.4.4.5.3. The enclosing capture result remains
unadmitted until those independent mandatory boundaries have their own evidence.
