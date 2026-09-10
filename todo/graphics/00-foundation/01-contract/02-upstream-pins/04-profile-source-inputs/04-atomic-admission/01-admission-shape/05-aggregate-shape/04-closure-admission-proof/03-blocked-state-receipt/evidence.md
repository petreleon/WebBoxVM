# F02.4.4.1.5.4.3 evidence

Revision: 958b53e811eb2cd0762400c511fc2d8086a0588b
Validation: blocked receipt 4/4 + roadmap tests 10/10 + roadmap limits + diff + make test
Result: PASS
Artifacts: validator sha256=6d73eceda4e3d8833803b3f8f575c62993f82bc79918a829e14d9370e625428d;
receipt JSON sha256=1b15139349b59c05830707a5b2283be4dd5db823231ce874e7c4b436c4b72660;
tests sha256=1278bba4388f41de796be2a552822e5f42663318ff3a94be82b607708556bccf
Profile: proof-of-correctly-blocked-admission-state; no guest API, inventory, cache, cutover, or F03 implementation

Task ID and date: F02.4.4.1.5.4.3, 2026-09-11 Europe/Bucharest.
Tested commit and dirty diff hash: `958b53e811eb2cd0762400c511fc2d8086a0588b` is the tested feature
commit; this receipt and its two checklist links are the uncommitted receipt-only diff.
Upstream manifest revision: the receipt rebuilds the six roles from source-requirements raw SHA-256
`a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5` and inventory lock
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.
Guest image and build hashes: not applicable; no guest image is built or run.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution route exists here.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 with `PYTHONDONTWRITEBYTECODE=1` ran `blocked_state_receipt.py`, `--build`, and its test suite;
`scripts/test_check_graphics_roadmap.py`, `scripts/check_graphics_roadmap.py`, `git diff --check`, and
`make test` ran with Cargo 1.93.0 and Node v26.0.0.
Expected result and minimum nonzero case count: one bounded JSON receipt reconstructs exactly one
six-role `blocked` state with three ordered blockers, five false readiness flags, and three false V2 flags.
Actual passed/failed/skipped counts and exit codes: receipt suite 4/0/0 and roadmap tests 10/0/0;
roadmap limits were valid; every command exited 0. `make test` reported Cargo 1,127 passed, 0 failed,
3 ignored, and Node 337 passed, 0 failed, 0 skipped.
Negative/reference checks and observed output: raw self-hash, duplicate, oversize, and extra-schema JSON
fail; re-sealed readiness/state/scope/blocker/digest/bridge changes fail type-aware exact comparison; stale
V1 and V2 inputs fail native reconciliation. The CLI prints
`RECEIPT: blocked 3 blockers 9ff372bcfeb83538882da396b2ecbb0801703ca55a1b0dd38fa7ab51d4bf501a`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no payload, image, or raw
log is retained. Re-run the cited Python suite beside the three hashed artifacts above.
Software fallback detection and actual execution route: not applicable; no renderer or fallback runs.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: canonical first blocker remains
`gles-cts-manifest: requires-multifile-core-selector-closure`; Docs and VCTS blockers remain material.
Decision and limits of the evidence: PASS means only the diagnostic receipt is bounded, self-hashed,
and exactly rebuilt from native V1/V2 validation. Its snapshot proves no change to source map, manifest,
`inventory.lock`, inventory inputs, F03 requirements/profile scope, audits, or V2 receipts. The receipt
records V1/V2 input and validator digests, selector–commit–root bridge, 98 V2 members / 434,669,348 bytes,
and `admitted=false`, `cutover_ready=false`, `satisfies_vulkan_14_core_manifest=false`. It neither admits
a source nor creates an inventory, fresh cache proof, `admission_closures.json`, `cutover_receipt.json`,
cache marker, F03 state, guest API, browser compatibility, CTS conformance, or performance claim.
Commit/push verification: feature commit `958b53e811eb2cd0762400c511fc2d8086a0588b` is local. Remote
publication and CI remain unrun because the environment rejected the prior push attempt; new explicit
publication authorization is required.
Next ready task: F02.4.4.1.5.4.4 — establish an admission-eligible closure (currently materially blocked).
