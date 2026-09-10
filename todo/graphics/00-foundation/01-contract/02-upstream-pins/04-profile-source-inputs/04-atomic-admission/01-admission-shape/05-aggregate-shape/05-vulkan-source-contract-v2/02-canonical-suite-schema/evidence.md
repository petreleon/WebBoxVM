# F02.4.4.1.5.5.2 evidence

Revision: f8bbaf76b3398290bf22794c2aef94c6da355748
Validation: schema 4/4 + F02/VCTS 15/10/9 + checker 10/10 + roadmap + diff + make test
Result: PASS
Artifacts: identity sha256=73dc13d423fe7da3d1f9f12283d0ae3093bce98827c886f75703dc5c0dac266d;
ledger sha256=e644b775601dd6a837c57ca5f35c2d86481d4b2df183f6bcaa0a9611791aa26f;
tests sha256=050c11a9ba6588f3e96a12653515101466c2a2bb60bfa0b745c6e59ca2001554;
root sha256=c2fc1ee3bb8113f71c50b9da94409aba6a7dd96cd59e3a3eed508b40f701c5b3
Profile: planning-only V2 Vulkan CTS source-contract schema; no guest, CTS, browser, or performance run

Task ID and date: F02.4.4.1.5.5.2, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `f8bbaf76b3398290bf22794c2aef94c6da355748`; the tree was clean.
Upstream manifest revision: annotated `vulkan-cts-1.4.6.2` tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`; `vk-default.txt` SHA-256
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`, 3,347 bytes.
Guest image and build hashes: not applicable; this task validates source metadata only.
Browser, OS, adapter and driver: not applicable; no browser or GPU route was exercised.
Exact command(s), working directory and tool versions: from `/Users/petreleon/code/WebBoxVM`, Python
3.14.6 ran `canonical_suite_contract_test.py`, `canonical_suite_identity.py`, the F02 fetch test,
VCTS candidate audit, VCTS boundary test, checker tests, and roadmap checker; Cargo 1.93.0 and Node
v26.0.0 ran `make test`.
Expected result and minimum nonzero case count: the V2 schema passes 4 test methods while rejecting
substituted/mutable/filtering roots, unsafe paths, root-only/duplicate/cyclic/reordered closures,
mixed or zero hashes, stale hashes, and numeric limits; existing 15/10/9/10 suites pass.
Actual passed/failed/skipped counts and exit codes: V2 4/0/0, F02 15/0/0, VCTS audit 10/0/0,
VCTS boundary 9/0/0, and checker 10/0/0, all exit 0. `make test` exited 0: Cargo reported 1,127
passed, 0 failed, 3 ignored; Node reported 337 passed, 0 failed, 0 skipped. Source limits passed.
Negative/reference checks and observed output: V2 asserts the exact ordered 98-path expansion derived
from the unchanged V1 `vk-default` transcript, root digest `30b272f8...c7bd5218`, and direct-path
digest `9f0e41be...8d37f7b9`; a self-resealed arbitrary, partial, or re-ordered selector fails.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no remote payload or GPU
capture was retained. Reproduce with the commands above; exact script/root hashes are recorded above.
Software fallback detection and actual execution route: not applicable; no guest/browser execution ran.
Performance conditions and frozen protocol version, when applicable: not applicable; no workload ran.
First failing subcheck or blocker, when applicable: no focused check failed. The external 98-member
payload/byte closure is intentionally not claimed here and remains the next task.
Decision and limits of the evidence: the immutable root is a regular F02.2 source; its larger closure is
a separate V2 type bounded at 128 members, 64 MiB per member, and 512 MiB aggregate. This proves only
the schema and selector identity, not cache bytes, Vulkan core compatibility, CTS conformance, Khronos
certification, browser execution, or near-native performance.
Commit/push verification: `f8bbaf76b3398290bf22794c2aef94c6da355748` was pushed to
`origin/codex/graphics-f01-baseline` and verified by `git ls-remote`; remote CI was not run.
Next ready tasks: F02.4.4.1.5.5.3 — verify the external closure cache; F02.4.4.1.5.5.4 —
freeze the coverage-report taxonomy.
