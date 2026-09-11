# F02.4.4.1.5.4.4.5.2.2.1 evidence

Revision: 134e03edd7ba213947f229bed09c9156bd8edccf
Validation: focused 3/3, predecessor checks, source limits, roadmap checker, diff, and make test
Result: PASS
Artifacts: contract self-hash 4a9369fe82f427a61e6b8c402af30abf1cbffe6f5590f7a17b273c86ba5ae772
Profile: design-only GLES 3.2 successor closure contract; no fetch, cache write, source admission, guest, browser, CTS, conformance, or performance run

Task ID and date: F02.4.4.1.5.4.4.5.2.2.1, 2026-09-11 Europe/Bucharest.

Tested commit and documentation diff: 134e03edd7ba213947f229bed09c9156bd8edccf is the exact
tested feature revision. It follows d6d96db4, whose separate cache-grammar correction is included in
the tested state; this receipt and checklist update are documentation-only afterward.

Pinned source identity: commit 067e8832315e79817ede1c4863804e440f5d1c80 binds the generated
[GLES root](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/067e8832315e79817ede1c4863804e440f5d1c80/external/openglcts/data/gl_cts/data/mustpass/gles/khronos_mustpass/main/mustpass.xml),
four ordered core selector files, 12 configurations, and one explicit optional-extension exclusion.
The contract records the same commit's build_mustpass.py and mustpass.py as producer entrypoints; it
expressly does not assert a producer execution or output attestation.

Contract invariants: each root/member URL is pinned to that revision; the root plus four core members
and excluded extension use Apache-2.0 repository terms, exact digest/byte identities, and the F02.2
8,388,608-B cap. The six cache identities use exactly webboxvm-graphics/f02/{id}/{sha256}.source, the
existing F02 grammar. Grammar compatibility is not a cache write, fresh-cache proof, atomic store, or
offline replay. Active schema-v2's 17-family lock, its inventory, and all support effects stay false.

Guest image and build hashes: not applicable; this task does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task does not execute a renderer or browser.
Software fallback and execution route: not applicable; no execution route exists.
Performance conditions and frozen protocol: not applicable; no workload runs.

Commands and results: from /Users/petreleon/code/WebBoxVM, Python 3.14.6 ran
gles_successor_contract_test.py (3 passed, 0 failed), the multi-suite integration test (3/0/0),
the independent-suite boundary test (3/0/0), GLES candidate audit (9/0/0), and F02 fetch-policy test
(15/0/0). Cargo 1.93.0 ran the source-limit test (6/0/0). make test reported 1,127 Rust passed /
0 failed / 3 ignored and Node 337 passed / 0 failed. git diff --check was clean; every command exited 0.
The final roadmap checker accepted 337 documents, 202 tasks, 65 PASS-complete, and 27 superseded.

Expected result and minimum nonzero case count: one self-hashed record must freeze six physical
identities, one exclusion, 12 core configurations, and every false effect. At least three focused
methods must prove it is read-only and reject re-sealed identity, cache, scope, and promotion changes.

Negative checks: the focused suite rejects changed revision, omitted core member, root byte drift,
non-F02 cache grammar, missing cache identity, output-attestation claim, fresh-cache claim, active-v2
mutation, admission promotion, stale self-hash, duplicate keys, oversized input, FIFO, and symlink.

Reproduction artifacts: gles_successor_contract.py SHA-256
ecad09d43f963caa69afd2b7070148244e11b5a8bbbbb4aaaccb5990ce444561; record SHA-256
5621df8625cee3c377c03df59748657db387f088f38bd1694073dc3da6ce24ef; test SHA-256
84a0208ed823fe11fc533874bf0d330f8c1cd34f0b9e3f937c76f307ea385ab7. No payload, capture, or
durable raw log is retained; re-run the focused suite beside these files.

First failing subcheck or blocker: this design task has no failing subcheck. The next mandatory
condition is F02.4.4.1.5.4.4.5.2.2.2: fetch the frozen six identities, prove atomic storage, and replay
the full closure offline. It is a local implementation obligation, not a claim that Khronos lacks a
root/member/producer description.

Decision and limits: PASS proves only a bounded, hostile-tested source-contract design. It does not
fetch a payload, write a cache, prove freshness, admit a source, change F03, run CTS, expose a guest
API, execute in a browser, establish Khronos certification, or measure performance.

Commit/push verification: d6d96db4 and fix 134e03ed are local on codex/graphics-f01-baseline.
No remote push or CI run occurred; publishing requires fresh explicit authorization.

Next ready task: F02.4.4.1.5.4.4.5.2.2.2 captures/replays this exact closure. Independently,
F02.4.4.1.5.4.4.5.3 remains blocked on a Khronos immutable explicit Vulkan 1.4 core VCTS manifest.
