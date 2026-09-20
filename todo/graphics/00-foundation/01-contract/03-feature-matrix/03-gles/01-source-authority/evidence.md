# F03.3.1 evidence

Revision: `13931d12fe0f50eded97fe8669a8d984c186207b`
Validation: `make graphics-gles-source-authority-test`; `make test`; `git diff --check`; `python3 scripts/check_graphics_roadmap.py`
Result: PASS
Artifacts: `gles_source_authority.json` SHA-256 `bd946472e204ef689278ff0bf0ac8d0c65435d028133e2a54a098021ad3616a6`
Profile: GLES 3.2 source-boundary only; `blocked` / `matrix-incomplete`

Task ID and date: F03.3.1, 2026-09-21.

The fixed-path F03 role-aware binding raw-lock-loads the sealed F02 contract
`d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3`.
No historical manifest, `gl.xml`, or ESSL alias is consumed.

The six focused tests cover the exact GLES normative/full-suite roots, self-hash,
stale/mixed/reordered/legacy/auxiliary/wrong-profile inputs, stale raw lock, ambient
modules, invalid locators, and attempted shader/precision/extension consumption.
They pass 6/6. The CLI reports two derivable and three unavailable classes.

`command-state` and `limit-format` may use only
`gles32-pdf-v1:page=<positive-decimal>;section=<section-path>` locators from the
sealed GLES normative PDF. The normative source directs shader-language and precision
semantics to a companion ESSL document; that document is not admitted. `shader`,
`precision`, and `extension` therefore retain `unadmitted-distinct-source` blockers.

No external capture, guest execution, browser execution, renderer invocation, or CTS
case execution occurred. Every qualification claim remains false and
`cts_executions` remains zero. The next leaf may consume only the two derivable
classes; separate F02 admission is required for the three unavailable classes.

Commit/push verification: source-boundary implementation committed and pushed as
`9af5335bcf610b9024588682de565e4ed689c0ea`; `git ls-remote --heads origin
codex/graphics-f01-baseline` matched that revision.
