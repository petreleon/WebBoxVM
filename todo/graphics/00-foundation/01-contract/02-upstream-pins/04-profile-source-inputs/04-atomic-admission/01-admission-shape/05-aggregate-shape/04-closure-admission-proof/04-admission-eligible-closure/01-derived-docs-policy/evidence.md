# F02.4.4.1.5.4.4.1 evidence

Revision: `5b78750651bd30ba0168d87cd3d8eb3f27203986` atop `7e7f888c` and `364e96c9`
Validation: focused policy 5/5, source limits 6/6, roadmap tests 10/10, roadmap checker, diff, and `make test`
Result: PASS
Artifacts: policy sha256=`86abe34a6c9c9b08707a1b85cc75e9366af5eca1bb90d3f68a86e7eb8d23f4e7`;
validator sha256=`3627c0c81d6b199678c97ef88b770517611848b686267cc1e9ec514602600858`;
tests sha256=`488ddbee1ebc45a2100a6a2344a03f4a85426d3b5e5757b5ae91ece6073b6e00`
Profile: policy-only Vulkan-Docs successor boundary; no active source admission, guest API, renderer, CTS, or performance result

Task ID and date: F02.4.4.1.5.4.4.1, 2026-09-11 Europe/Bucharest.

Tested commit and dirty diff hash: feature commits `7e7f888c`, `364e96c9`, and
`5b78750651bd30ba0168d87cd3d8eb3f27203986` were tested with a clean worktree. This receipt and
the checked roadmap links are receipt-only changes after that feature revision.

Upstream manifest revision: the policy locks Vulkan-Docs revision
`f84d432d5b8912362f96f581f29bbc4f3c8c7843`, raw root `vkspec.adoc`, root SHA-256
`069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0`, and the reviewed F03
source-requirements SHA-256 `a9535611bab654034357d1578c3a2035caa09fda8090114df0867ac6d7fbeca5`.

Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task performs no browser or GPU execution.

Exact commands, working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 ran `derived_docs_policy_test.py`, `derived_docs_policy.py`,
`scripts/test_check_graphics_roadmap.py`, and `scripts/check_graphics_roadmap.py`; Cargo 1.93.0 ran
`cargo test -p emulator --test source_file_limits --quiet` and `make test`; Node v26.0.0 ran the
browser lane through `make test`; `git diff --check` checked the final diff.

Expected result and minimum nonzero case count: one self-hashed policy must retain the exact F03 role,
raw-tree cap, pinned core build route, separated generated-output cap/license obligations, and false
effects. At least five focused test methods must reject tampering or prove the unadmitted policy record.

Actual passed/failed/skipped counts and exit codes: focused policy suite 5/0/0; source-file limits
6/0/0; roadmap tests 10/0/0. The checker accepted 314 documents, 188 tasks, 57 PASS-complete and
27 superseded tasks before this receipt update. Every command exited 0. `make test` passed Cargo
1,151 cases with 0 failures and 3 ignored, and Node 337/337 with 0 failures or skips.

Negative/reference checks and observed output: re-sealed changes to the role, schema, raw cap,
revision, scope witness, build route, license obligation, VCTS flags, and all false effects fail. The
suite also rejects `true`/`1`, `false`/`0`, and integer/float aliases; duplicate and oversized JSON;
and FIFO or symlink policy paths. The CLI prints
`POLICY: f84d432d5b8912362f96f581f29bbc4f3c8c7843 raw-cap=8388608 unadmitted`.

Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no image, guest payload,
or durable raw log is retained. Re-run the cited focused suite beside the three hashed policy files;
the validator rehashes the F03 requirement and all three retained witness/scope/comparison anchors.

Software fallback detection and actual execution route: not applicable; no rendering route runs.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload runs.

First failing subcheck or blocker, when applicable: the successor policy itself has no failing
subcheck. Aggregate admission remains blocked first by
`gles-cts-manifest: requires-multifile-core-selector-closure`; the Docs closure and VCTS core-manifest
truth remain independent requirements.

Decision and limits of the evidence: PASS means only that the policy envelope is bounded, pinned,
type-strict, self-hashed, and hostile-tested. It reserves a future reproducible Vulkan-Docs core
closure while forbidding root-only, mutable, output-as-source, VCTS-as-Docs, and false-ready records.
It does not create a Docs closure, mutate the active 17-family inventory, prove cache freshness, alter
F03, advertise an API, establish guest compatibility, CTS conformance, Khronos certification, or
near-native performance.

Commit/push verification: the tested feature commits are local on `codex/graphics-f01-baseline`.
Remote publication and CI are unrun because prior push authorization was rejected; a new explicit
authorization is required before pushing.

Next ready task: F02.4.4.1.5.4.4.2 — prove the derived-Docs source closure.
