# F02.4.4.1.5.5.1 evidence

Revision: 0d61171b3369b98b7e637c04d304f04fef042fea
Validation: checker 10/10 + F02/VCTS 15/10/9 + source limit 6/6 + roadmap + diff + make test
Result: PASS
Artifacts: checker sha256=a3cf186ad8b7c16baf733af9f811f4a6d76e64e01f15a0f54c97fa7776498b06;
checker-tests sha256=cbf51f357620ed34c92d238d181db4caf710b5b7e56fa5f2d59524019ab1a56f;
V2-plan sha256=7c1eb9f1b19e08f6cd11f9365986d596c69714063936ca16a5b75c0053271ee0
Profile: planning-only Vulkan 1.4 core source-contract decision; no guest, CTS, browser, or performance run

Task ID and date: F02.4.4.1.5.5.1, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `0d61171b3369b98b7e637c04d304f04fef042fea`; the tree was clean.
Upstream manifest revision: no source payload changed. The decision retains the audited Vulkan Docs
`v1.4.362`/`f84d432d5b8912362f96f581f29bbc4f3c8c7843` and VCTS
`vulkan-cts-1.4.6.2`/`f6a29701220f34dd1407513bfe80d74ca7b392ce` identities.
Guest image and build hashes: not applicable; this is a source-policy decision with no guest build.
Browser, OS, adapter and driver: not applicable; no browser or GPU route was exercised.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 ran `scripts/test_check_graphics_roadmap.py`, the F02 fetch test, VCTS candidate audit, VCTS
boundary test, and `scripts/check_graphics_roadmap.py`; Cargo 1.93.0 and Node v26.0.0 ran `make test`.
Expected result and minimum nonzero case count: 10 checker, 15 fetch, 10 VCTS audit, and 9 boundary
cases pass; the roadmap exposes V2 policy decision as the only ready leaf without classing it as support.
Actual passed/failed/skipped counts and exit codes: checker 10/0/0, fetch 15/0/0, VCTS audit 10/0/0,
and boundary 9/0/0, all exit 0. `make test` exited 0: Cargo reported 1,127 passed, 0 failed, 3 ignored;
Node reported 337 passed, 0 failed, 0 skipped. Its source-limit suite reported 6/0/0.
Negative/reference checks and observed output: checker fixtures reject a missing or superseded successor,
an active dependency on a superseded task, and an invalid historic PASS receipt. Existing VCTS tests
preserve the rejected V1 root-only/core-only/over-limit boundary. Roadmap output was `290 documents,
176 tasks, 46 PASS-complete, 27 superseded`, with this decision leaf initially the sole ready task.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no remote payload or GPU
capture was retained. Reproduce with the commands above; the maintained checker and V2-plan digests
are recorded in Artifacts.
Software fallback detection and actual execution route: not applicable; no graphics implementation ran.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused check failed. V1 remains historical:
generated Docs lack authoritative per-member license/role/provenance, and VCTS lacks an upstream
core-only selector while the V1 per-member 8 MiB rule rejects the canonical suite closure.
Decision and limits of the evidence: the user approved V2. `regular-source` retains F02.2's 8 MiB
policy; `canonical-upstream-suite` binds an immutable Khronos root and external closure receipt; generated
Docs are non-distributable provenance material unless authority is later established. Core readiness and
complete `vk-default` diagnostics are separate reports. This is not a compatibility, CTS conformance,
Khronos certification, browser execution, or near-native-performance claim.
Commit/push verification: `0d61171b3369b98b7e637c04d304f04fef042fea` was pushed to
`origin/codex/graphics-f01-baseline` and verified by `git ls-remote`; remote CI was not run.
Next ready task: F02.4.4.1.5.5.2 — define the canonical-suite schema.
