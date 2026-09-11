# F02.4.4.1.5.4.4.5.1 evidence

Revision: b1224b5e178a23059772860e88cc1cb5e8c6d4a9
Validation: focused 3/3, native GLES/VCTS/reconciler/receipt checks, source limits, checker, diff, and `make test`
Result: PASS
Artifacts: policy self-hash da10dbc51230b2495f7c24c100055ab91970a92217b43f2895ec5cfa2ead459b
Profile: policy-only independent GLES/VCTS successor boundary; no source admission, guest, renderer, CTS, or performance run

Task ID and date: F02.4.4.1.5.4.4.5.1, 2026-09-11 Europe/Bucharest.

Tested commit and dirty diff hash: `b1224b5e178a23059772860e88cc1cb5e8c6d4a9` was tested as
the exact feature content before its local commit. The worktree was clean after that commit; this
receipt and checklist update are the subsequent documentation-only diff.

Current immutable boundary: GLES candidate `gles-cts-manifest` remains rejected with blocker
`requires-multifile-core-selector-closure`. Its four known core selectors total 875,324 B, run
12,477 cases in 12 configurations / 30,574 case-configuration runs, and retain one explicit
optional extension exclusion. They are only an unadmitted logical closure, not a fresh successor.

VCTS boundary: the complete immutable `vk-default` diagnostic remains 98 ordered members / 434,669,348 B,
with taxonomy `0 core / 1 WSI / 1 video / 4 extension / 92 unknown`. Fourteen members exceed F02.2's
8,388,608-B per-source cap. Its V2 64-MiB diagnostic member limit is not a source-policy exception;
`local_filtering=forbidden` and `satisfies_vulkan_14_core_manifest=false` remain exact.

Guest image and build hashes: not applicable; this policy does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this policy does not execute a browser or renderer.
Software fallback detection and actual execution route: not applicable; no execution route exists.
Performance conditions and frozen protocol version: not applicable; no workload runs.

Exact commands, working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 ran `suite_successor_boundary_test.py`, GLES `candidate_audit_test.py`, VCTS
`vcts_aggregate_handoff_test.py`, `current_evidence_reconciler_test.py`,
`blocked_state_receipt_test.py`, and `scripts/check_graphics_roadmap.py`; Cargo 1.93.0 ran
`cargo test -p emulator --test source_file_limits --quiet`, `cargo test -p emulator --quiet`, and
`make test`; Node v26.0.0 ran the Node lane through `make test`; `git diff --check` was clean.

Expected result and minimum nonzero case count: one self-hashed policy must retain both exact blocked
forms, the raw byte anchors, F02.2's 8 MiB cap, and every false effect. At least three focused test
methods must prove the policy is read-only and reject re-sealed promotion or unsafe JSON inputs.

Actual passed/failed/skipped counts and exit codes: focused boundary 3/0/0; GLES 9/0/0; VCTS 8/0/0;
reconciler 4/0/0; blocked receipt 4/0/0; source limits 6/0/0. Every command exited 0. The checker
accepted 329 documents, 197 tasks, 62 PASS-complete and 27 superseded. Cargo reported 1,127 passed,
0 failed, 3 ignored in its main 1,130-test suite; `make test` also reported Node 337 passed / 0 failed.

Negative/reference checks and observed output: re-sealed admission, conformance, closure-ready,
core-manifest-ready, local-selector, taxonomy-as-conformance, core-category, over-cap, scope, extension,
and stale-anchor mutations fail. The focused suite also rejects invalid self-hashes, duplicate and
oversized JSON, FIFO, and symlink inputs. CLI output is
`POLICY: policy-only-unadmitted da10dbc51230b2495f7c24c100055ab91970a92217b43f2895ec5cfa2ead459b`.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no guest payload, browser
capture, or durable raw log is retained. Re-run the cited focused suite beside policy source SHA-256
`5b21324cc01fe90914bfcd80df5a834649f076fc5668099b24449c3b1180ebeb`, record SHA-256
`6a0a7fc4562acb63678ccfc2d6c965355bcce7b417a0483734119c84698025ff`, and test SHA-256
`b796e4a22b27228df49354591b0f0dc9fcca4e1a2ca71e3858f364f6b75b8351`.

First failing subcheck or blocker: this policy task has no failing subcheck. The global first blocker
remains `gles-cts-manifest: requires-multifile-core-selector-closure`; a Khronos-published immutable
Vulkan 1.4 core manifest is independently still absent.

Decision and limits of the evidence: PASS proves only a bounded, self-hashed, hostile-tested policy
boundary. It creates no successor closure, inventory, cache freshness, F03 change, admission, cutover,
guest API, browser behavior, CTS result, Khronos certification, or performance claim.

Commit/push verification: feature commit `b1224b5e` is local on `codex/graphics-f01-baseline`; before
this receipt commit, the branch was 23 commits ahead of `origin/codex/graphics-f01-baseline`. No remote
push or CI run occurred: a fresh explicit authorization is required before publishing.

Next ready task: F02.4.4.1.5.4.4.5.2 and F02.4.4.1.5.4.4.5.3 remain separate external-proof tasks;
F02.4.4.1.5.4.4.2.2 remains blocked by Docs per-member authority.
