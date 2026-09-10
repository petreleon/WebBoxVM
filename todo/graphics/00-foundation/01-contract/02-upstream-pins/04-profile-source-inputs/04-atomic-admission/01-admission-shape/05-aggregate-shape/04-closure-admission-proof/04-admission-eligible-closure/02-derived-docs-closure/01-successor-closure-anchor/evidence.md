# F02.4.4.1.5.4.4.2.1 evidence

Revision: f688fd2c2c29ed71f172a9c9871e37869b79a7fa
Validation: focused anchor 5/5, source limits 6/6, roadmap tests 10/10, checker, diff, and make test
Result: PASS
Artifacts: anchor sha256=18635158b68d98665006d3420faed5341f66b4d7591ee70ff525e8538f6f1d5d;
validator sha256=795b2261242473f49332e506027802be959d74756508f96aed9e91c0d96a6a63;
tests sha256=5f7fe58bb4f7e4393e754a37ad8474253e14f8b591beabf422e6615a53b7fee9
Profile: policy-bound historical Vulkan-Docs evidence; no fresh closure, source-role admission, guest API,
renderer, CTS, or performance result

Task ID and date: F02.4.4.1.5.4.4.2.1, 2026-09-11 Europe/Bucharest.

Tested revision and dirty-diff state: feature revision f688fd2c2c29ed71f172a9c9871e37869b79a7fa
was tested before this receipt-only diff. The feature worktree was clean; this receipt and its roadmap
links do not alter the anchored policy or witnesses.

Pinned input and scope: the adapter verifies the self-hashed derived-Docs policy at Vulkan-Docs revision
f84d432d5b8912362f96f581f29bbc4f3c8c7843. It reads each retained historical JSON once as bounded raw
bytes, verifies that exact byte digest against the policy anchor, then parses the same bytes. The exact
target is Vulkan 1.4 core, api-limit-format-spec, and vulkan-14-spec.

Historical facts retained: 298 raw members, 1,462 derived members, two matching input observations, and
2,530 output-tree files. The one rendered HTML output is separately classified as 10,377,052 bytes; it
cannot satisfy a source role. The build, scope, and comparison statuses remain unadmitted.

Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; this task performs no browser or GPU execution.

Commands and results: from /Users/petreleon/code/WebBoxVM, Python ran successor_closure_anchor_test.py
and successor_closure_anchor.py; Cargo ran cargo test -p emulator --test source_file_limits --quiet;
Python ran scripts/test_check_graphics_roadmap.py and scripts/check_graphics_roadmap.py; git diff --check
ran; and make test ran the full Cargo and Node lanes. All commands exited 0. The focused suite passed
5/0/0, source limits 6/0/0, roadmap tests 10/0/0, and the Node lane passed 337/337 with no failures.

Negative/reference checks: the focused suite rejects stale policy or historical-byte anchors, duplicate or
oversized JSON, FIFO or symlink policy paths, altered target/count/status/output/configuration records,
output-as-source assertions, and every true or type-aliased false-ready flag. The accepted CLI output is:
ANCHOR: historical 298 raw 1462 derived unadmitted.

Raw log/image/sample paths and reproduction: no image, guest payload, or durable raw log is retained.
Run the cited focused suite beside the three hashed anchor files. The validator rechecks policy and
historical byte identities before checking the parsed records.

Fallback detection and performance conditions: not applicable; no renderer or workload runs.

First failing subcheck or blocker: this anchoring task has no failing subcheck. The next Docs blocker is
authoritative per-member license, attribution, source-role, provenance, and producer authority for all
1,462 derived members, followed separately by fresh authorized write lineage. Aggregate admission remains
blocked first by gles-cts-manifest: requires-multifile-core-selector-closure.

Decision and limits: PASS means only that the old evidence is accurately bound and cannot be relabeled
fresh, source-valid, admitted, cutover-ready, supported, conformant, certified, or near-native. It does
not create the two fresh isolated replays, change F02 inventory or F03, prove compatibility, advertise a
guest API, or make any Khronos claim.

Commit/push verification: f688fd2c is local on codex/graphics-f01-baseline. Remote publication and CI
remain unrun: a fresh explicit authorization is required before a push after the prior rejection.

Next ready task: F02.4.4.1.5.4.4.2.2 — establish derived-member authority.
