# F02.4.4.1.5.4.2 evidence

Revision: d9cfc410403e3bfded8225dc3604775b5e23ace4
Validation: reconciler 4/4 + roadmap tests 10/10 + roadmap limits + diff + make test
Result: PASS
Artifacts: reconciler sha256=ef946aef7ebccabfc97de409aa59491dc748fc78c4c7b0cf446eeec49a451a71;
tests sha256=192431efcae57e9c143762f9212eaa0827c3cbaabe86162221928d71cda7373b
Profile: proof-of-correctly-blocked-admission-state; no guest API, cache, inventory, cutover, or F03 implementation

Task ID and date: F02.4.4.1.5.4.2, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `d9cfc410403e3bfded8225dc3604775b5e23ace4` is the tested feature commit; this receipt and its two
checklist links are the uncommitted receipt-only diff, with no source implementation change after testing.
Upstream manifest revision: the six frozen F03 roles retain requirements raw SHA-256
`a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5` and inventory lock
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
Guest image and build hashes: not applicable; this stdlib-only reconciler executes no guest image.
Browser, OS, adapter and driver: not applicable; this is no browser or GPU execution route.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 with `PYTHONDONTWRITEBYTECODE=1` ran `current_evidence_reconciler.py` and its test suite;
`scripts/test_check_graphics_roadmap.py`, `scripts/check_graphics_roadmap.py`, `git diff --check`, and
`make test` ran with Cargo 1.93.0 and Node v26.0.0.
Expected result and minimum nonzero case count: one valid six-role result is `blocked`, never
`admission_eligible`; the focused suite has four test methods and seven independently resealed V2 hostile cases.
Actual passed/failed/skipped counts and exit codes: reconciler 4/0/0 and roadmap tests 10/0/0; roadmap
limits were valid; every command exited 0. `make test` reported Cargo 1,127 passed, 0 failed, 3 ignored, and
Node 337 passed, 0 failed, 0 skipped.
Negative/reference checks and observed output: hostile checks reject semantic-preserving V1 byte drift,
audit substitution, an admitted Docs row, erased GLES closure, root fallback, all seven V2 inputs after
resealing, V1-as-V2 substitution, detached V1/V2 selector–commit–root joins, and incomplete path sets.
The CLI prints `BLOCKED: 6 roles, first gles-cts-manifest requires-multifile-core-selector-closure`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no payload, image, or raw
log is retained. Re-run the cited Python suite beside the two hashed artifacts above.
Software fallback detection and actual execution route: not applicable; no renderer or fallback runs.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: the canonical first blocker is
`gles-cts-manifest: requires-multifile-core-selector-closure`; the remaining V1 blockers are the unresolved
Docs closure and broader/oversize VCTS default suite.
Decision and limits of the evidence: PASS means the read-only reconciliation protocol correctly proves
`blocked`. V1 locks bind rules, source map, boundaries, and all three audits; V2 locks bind handoff,
root/tree/ledger/cache/live/taxonomy identities; the bridge binds V1 selector, V1 revision, and root SHA.
The V2 diagnostic remains 98 members / 434,669,348 bytes with `admitted=false`, `cutover_ready=false`,
and `satisfies_vulkan_14_core_manifest=false`. Docs remains outside implementation closure. This does not
admit a source, prove a fresh cache, create inventory, `admission_closures.json`, `cutover_receipt.json`,
cache marker, or F03 state, nor claim guest API, browser compatibility, CTS conformance, or performance.
Commit/push verification: feature commit `d9cfc410403e3bfded8225dc3604775b5e23ace4` is local. Remote publication and CI remain unrun
because the environment rejected the prior push attempt; a new explicit publication authorization is required.
Next ready task: F02.4.4.1.5.4.3 — seal the blocked-state receipt.
