# F02.4.4.1.5.4.4.2.2 blocker record

Revision: 873bec9a0e31418a12a566ca3ba0f82bfc934ec2
Validation: focused authority gate 5/5, source limits 6/6, roadmap tests 10/10, checker, diff, and make test
Result: BLOCKED
Artifacts: gate sha256=2a89826560ac6f42e286cec39106f05222f64711fc409d46bdde45f0be4e7004;
validator sha256=161298d4c6a0f59095f9f2c8596321e5b6c7bcb0487230a97a735cd9a66fe4be;
tests sha256=b9402ab6174161b7b32bd8d62694f32928ff477454d2986e60880b04096c6ab1
Profile: external-authority discovery only; no fresh closure, source admission, guest API, renderer, CTS,
conformance, certification, or performance result

Task ID and date: F02.4.4.1.5.4.4.2.2, 2026-09-11 Europe/Bucharest.

Tested revision and dirty-diff state: feature revision 873bec9a0e31418a12a566ca3ba0f82bfc934ec2
was tested before this receipt-only diff. The feature worktree was clean; this record and its link leave
the task unchecked and do not alter the gate.

Pinned upstream finding: in Vulkan-Docs revision f84d432d5b8912362f96f581f29bbc4f3c8c7843,
[COPYING.adoc](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/COPYING.adoc)
distinguishes source, tool, and output terms and permits generated transient files to lack copyright.
[REUSE.toml](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/REUSE.toml)
contains no generated-tree pattern, and the pinned Git tree contains no tracked generated directory.

Build finding: [BUILD.adoc](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/BUILD.adoc)
and [makeSpec](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/makeSpec)
allow the pinned core 1.4 build recipe but do not emit per-output license, attribution, role, provenance,
selector/digest, or producer identity. [vkconventions.py](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/scripts/vkconventions.py#L139-L140)
only supplies a generic generated-from-vk.xml warning.

Observed derived scope: all 1,462 captured derived members have no SPDX/copyright header; 1,453 contain
only that generic warning and nine do not. Their sizes range from 89 to 768,158 bytes, so the missing
authority is not an 8 MiB-cap failure. No official record binds every derived selector, digest, bytes,
generation identity, license expression, attribution, source role, provenance, and producer authority.

Compact-registry boundary: pinned [xml/vk.xml](https://github.com/KhronosGroup/Vulkan-Docs/blob/f84d432d5b8912362f96f581f29bbc4f3c8c7843/xml/vk.xml)
is 3,309,653 bytes, carries Apache-2.0 OR MIT, and explicitly contains VK_VERSION_1_4. It is a useful
machine-readable registry input, not a substitute for the normative vulkan-14-spec source role or a CTS
selector.

Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task performs no browser or GPU execution.

Commands and results: from /Users/petreleon/code/WebBoxVM, Python ran
derived_member_authority_gate_test.py and derived_member_authority_gate.py; Cargo ran
cargo test -p emulator --test source_file_limits --quiet; Python ran
scripts/test_check_graphics_roadmap.py and scripts/check_graphics_roadmap.py; git diff --check ran; and
make test ran the full Cargo and Node lanes. Focused authority tests passed 5/0/0; source limits passed
6/0/0; roadmap tests passed 10/0/0; the checker accepted 320 documents, 192 tasks, 59 PASS-complete and
27 superseded; and make test exited 0 with Node 337/337 passing. The intentional CLI result is exit 2:
AUTHORITY-GATE: 1462 derived; external authority unavailable.

Negative checks: a local manifest, raw-to-derived or output-to-source inheritance, uniform labels,
aliases/defaults, altered anchor/count/field order, true or type-aliased false effects, duplicate or
oversized JSON, FIFO/symlink input, and bad CLI arity all fail. The gate is read-only.

First failing subcheck: no authoritative upstream source covers all five required fields for every
derived member. Only Khronos publishing an immutable, applicability-bound per-member manifest could
resolve this subcheck; a local reconstruction would fabricate authority.

Decision and limits: retain this task unchecked. The gate makes the blocker explicit and prevents a local
record from becoming a source, inventory, F03, support, conformance, certification, performance, or
release claim. It does not change the historical-anchor result or bypass the independent fresh write-lineage
and GLES CTS closure blockers.

Commit/push verification: 873bec9a is local on codex/graphics-f01-baseline. Remote publication and CI
remain unrun: a fresh explicit authorization is required before pushing after the prior rejection.

Current routing: no roadmap leaf is executable while this external authority and the independently
blocked VCTS manifest remain unavailable. Recheck this child only when Khronos publishes applicability-bound
per-member authority; do not treat this record as source admission or a substitute for the open work.
