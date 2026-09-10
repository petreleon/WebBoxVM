# F02.4.4.1.5.4.4.3 evidence

Revision: 189305db5ff9b283dabd1b0c1f92da92d0c0c152
Validation: focused boundary 6/6, source limits 6/6, roadmap tests 10/10, checker, diff, and make test
Result: PASS
Artifacts: boundary sha256=4544bc95c6021fd85d134eea5d4f2913d4fdaf810e9058d74d220dda7d1cdc40;
validator sha256=1384c19642023c7f003dab625ae55f0b7240197a4bfbf90e93f0a6a10275479b;
tests sha256=d801fc87b17990d51c260af342b406bcc1a5d2385d41307a05dee5545aca1a8a
Profile: source/release truthfulness boundary only; no active source closure, guest, renderer, CTS,
conformance, certification, or performance result

Task ID and date: F02.4.4.1.5.4.4.3, 2026-09-11 Europe/Bucharest.

Tested revision and dirty-diff state: feature revision 189305db5ff9b283dabd1b0c1f92da92d0c0c152
was tested before this receipt-only and roadmap-split diff. The feature worktree was clean; these docs
do not alter the self-hashed boundary.

Input identities: the boundary raw-byte anchors the six F03 role tuples, the derived-Docs policy, and
the V2 canonical-suite handoff. It fixes xml/vk.xml at Vulkan-Docs revision
f84d432d5b8912362f96f581f29bbc4f3c8c7843, SHA-256
cf31c965cf6e788697139601da0c7e02a75a9b6c7ac764e7641f5521ffd9da06, 3,309,653 bytes,
Apache-2.0 OR MIT (SPDX file notice), and VK_VERSION_1_4. This is compact registry metadata only.

Role boundary: raw Docs provenance cannot be replaced by generated Docs or the registry; full immutable
vk-default remains a Khronos default mustpass suite broader than Vulkan 1.4 core. Its core, WSI, video,
extension, and unknown categories are reports and may not filter the selector or become conformance.
The future source-contract v3 must bind every profile role and exact closure identity before any F03
transition remains possible.

Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task performs no browser or GPU execution.

Commands and results: from /Users/petreleon/code/WebBoxVM, Python ran source_release_boundary_test.py
and source_release_boundary.py; Cargo ran cargo test -p emulator --test source_file_limits --quiet;
Python ran scripts/test_check_graphics_roadmap.py and scripts/check_graphics_roadmap.py; git diff --check
ran; and make test ran full Cargo and Node lanes. All commands exited 0. Focused boundary tests passed
6/0/0, source limits 6/0/0, roadmap tests 10/0/0, and Node passed 337/337 with no failures.

Negative/reference checks: true and integer-alias effects, source or release promotion, registry-as-spec
or CTS, generated Docs as source, local vk-default filtering, category truncation, role omission,
anchor mutation, duplicate/oversized/symlink documents, bad CLI arity, and CLI writes all fail.

First failing subcheck or blocker: this boundary task has no failing subcheck. It deliberately preserves
the current blockers: per-derived Docs authority and authorized write lineage, and aggregate-first
gles-cts-manifest: requires-multifile-core-selector-closure.

Decision and limits: PASS means only that the channels and claims are rigorously separated. It does not
admit a source, change F02 inventory or F03, make a matrix row supported, run CTS, establish guest
compatibility, certify Khronos conformance, or measure near-native browser performance.

Roadmap change: F03.4 now has a registry-inventory child and a Docs-provenance/CTS-diagnostics child.
Their dependencies preserve the source-contract cutover; neither changes the current requirements file.

Commit/push verification: 189305db is local on codex/graphics-f01-baseline. Remote publication and CI
remain unrun: a fresh explicit authorization is required before pushing after the prior rejection.

Next independent ready task: F03.4.1 — enumerate the Vulkan registry inventory.
