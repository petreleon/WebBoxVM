# F02.4.4.1.5.4.4.6 evidence

Revision: 628650fce83d8efea1d947a23b3c4c7add6ff08b
Validation: focused 5/5, source limits 6/6, checker, diff checks, and make test
Result: PASS
Artifacts: transition self-hash d9317d418f6c13f4c85275bffb1a01139eb036e7ddf5853a347dde84d69273c5
Profile: design-only Vulkan 1.4 source-inventory transition; no source closure or guest execution

Task ID and date: F02.4.4.1.5.4.4.6, 2026-09-11 Europe/Bucharest.

Tested commit and dirty diff hash: 628650fce83d8efea1d947a23b3c4c7add6ff08b was tested from
the exact staged feature content, then committed. The worktree was clean immediately after that
commit; this receipt and checklist-only update do not alter the feature contract.

Design boundary: schema-v2 and its immutable 17-family lock
08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6 remain active. Schema-v3
is future-only: it preserves all 17 raw legacy records byte-equivalently, retains `vulkan-registry`,
and adds one `vulkan-docs` wrapper containing ordered raw members. It does not mutate F02.1/F02.2.

Pinned identities: the receipt binds the Docs policy document/self hashes
86abe34a6c9c9b08707a1b85cc75e9366af5eca1bb90d3f68a86e7eb8d23f4e7 /
9aac96fcdc340a31e77f44318fd32dbd673eb90551b5ac090d21d8b3a53078aa, the historical anchor,
and the source/release boundary. Its root is `vkspec.adoc` at raw SHA-256
069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0.

Future-only constraints: each raw member is capped at 8 MiB, has an immutable pinned HTTPS URL,
and uses `webboxvm-graphics/f02-successor/vulkan-docs/<closure>/<id>/<sha>.source`. A real cutover
requires a fresh complete root-and-nonroot closure, ordered member identities and closure digest,
per-member authority ledger, authorized write lineage, and atomic F02.1/F02.2 revalidation.

Shape fixture: the positive v3 sample is ephemeral and explicitly `shape-only-unadmitted`. Its
authority and lineage fields are null and freshness is false; it cannot serve as closure evidence.
The validator rejects a non-null authority/lineage/freshness claim, legacy-ID collision, family alias,
duplicate selector/cache/ID, root replacement, root-only import, unsafe selector/URL/cache, 8 MiB
overflow, self-hash drift, duplicate JSON keys, oversize/FIFO/symlink input, and effect promotion.

Guest image and build hashes: not applicable; this task does not build or run a guest.
Browser, OS, adapter and driver: not applicable; this task does not execute a renderer or browser.
Software fallback detection and actual execution route: not applicable; no execution route exists.
Performance conditions and frozen protocol version: not applicable; no workload runs.

Commands and results: from /Users/petreleon/code/WebBoxVM, ran
`PYTHONDONTWRITEBYTECODE=1 python3 .../successor_inventory_transition_test.py` (5 passed, 0 failed),
`cargo test -p emulator --test source_file_limits --quiet` (6 passed, 0 failed),
`PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` (PASS: 325 documents,
194 tasks), `git diff --check`, `git diff --cached --check`, and `make test`. Every command exited 0;
`make test` reported 1,127 Rust passed / 0 failed / 3 ignored and Node 337 passed / 0 failed.

Artifacts: `successor_inventory_common.py` SHA-256
b936bd2a213f116179145037d8bd6606dc82d9718c343651c2c1f2d0eb736925;
`successor_inventory_proposal.py` SHA-256
8e376d723e270280c2bc202889e874798f6eb4b747c006e61e5d7e2f2cac2024;
the receipt JSON raw SHA-256 was 492b900c82d40d2544b7f619f26e333d33eb79122cbef1d4502b8406ac5a09d3.

First failing subcheck or blocker: this design task has no failing subcheck. A real successor remains
blocked by missing authoritative per-derived Docs member ledger and fresh authorized write-lineage;
aggregate admission remains first blocked by `gles-cts-manifest: requires-multifile-core-selector-closure`.

Decision and limits of the evidence: PASS proves only a bounded, hostile-tested, immutable transition
design. It does not create a Docs closure, cache, or authority ledger; change F02/F03; expose Vulkan;
run CTS; establish guest compatibility or Khronos conformance; or measure browser performance.

Commit/push verification: feature commit 628650fc is local on `codex/graphics-f01-baseline`. Remote
publication and CI are unrun: a fresh explicit authorization is required before pushing.

Next ready task: F02.4.4.1.5.4.4.5 is ready only for a narrow policy split; its GLES and VCTS closure
obligations remain unresolved. F02.4.4.1.5.4.4.2.2 remains externally blocked by Docs authority.
