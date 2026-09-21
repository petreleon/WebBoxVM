# F03.4.2.3 evidence — root-wide VCTS diagnostics

Revision: uncommitted worktree based on `6539a7dab4579eba971882d98899eba9f009d7d9`.
Validation: fixed F02/F03 dependency suites, six focused hostile tests, external-cache replay, the
integrated `make test` target, source-file limits, diff, and roadmap checks below.
Result: PASS
Artifacts: [`vulkan_full_suite_diagnostics.json`](vulkan_full_suite_diagnostics.json), SHA-256
`d97b14dac6e9a5d804df0a6aba37c24d21d8170e6e692a90caf2f910ed7e8327`, embedded diagnostic SHA-256
`66d771723737b03f4a301db51b927ee51da89db9eaf5f2a10f8deeaf8e7f0b68` (67,926 bytes).
Profile: unfiltered Khronos `vk-default.txt` root only; no case selection, coverage, CTS, guest,
browser, conformance, certification, or performance result.
Status: source diagnostic only; Vulkan 1.4 remains `blocked` / `matrix-incomplete`.

Task ID and date: F03.4.2.3, 2026-09-21 Europe/Bucharest.

## Receipt

The diagnostic loads the exact F03 `vulkan-cts-default` full-suite role and F03.4.2.1 source-channel
boundary privately, then validates the retained external nine-file selector cache at
`/private/tmp/webboxvm-f0341.cqT6ZX`. It records the F02.5.3.3 immutable root revision, digest, bytes,
Apache-2.0 license, attribution, full ledger identity, taxonomy, and all 98 ordered member relations.

The member set totals 434,669,348 bytes. Its observed categories are 0 core, 1 WSI, 1 video,
4 extension, and 92 unknown. Every member is explicitly `root-wide-diagnostic-only`, with null
implementation owner and independent test plan, and `coverage: unassigned`. The serialized artifact
is below 8 MiB; its row-order digest is
`47fc6c455ad1bfd42c105ad8b7e4c3d4c6ab54ea962cbd28f2fc339a56f7f6af`.

All local qualification claims are false, CTS executions are zero, and the diagnostic has zero matrix
rows. The upstream root remains broader than a Vulkan-1.4-core selector; local filtering, case
selection, coverage inference, and CTS execution are forbidden.

## Verification

- `make graphics-vulkan-ledger-taxonomy-test graphics-vulkan-cache-replay-test graphics-vulkan-full-suite-receipt-test graphics-profile-source-gate-test graphics-vulkan-source-channel-boundary-test` — PASS: 4, 5, 6, 15 + 9, and 6 tests.
- `make graphics-vulkan-full-suite-diagnostics-test` — PASS: 6 hostile tests, including an arbitrary
  working-directory and `sys.path` sibling-decoy import.
- `PYTHONDONTWRITEBYTECODE=1 python3 .../vulkan_full_suite_diagnostics.py --selector-cache-root /private/tmp/webboxvm-f0341.cqT6ZX` — PASS: 98 root-wide diagnostic rows; cache identity revalidated.
- `wc -c .../vulkan_full_suite_diagnostics.json` — 67,926 bytes, below 8 MiB.
- `make test` — PASS: graphics targets, 1,127 Rust tests (3 ignored), source-file limits, and 338 Node tests.
- `cargo test -p emulator --test source_file_limits --quiet`, `git diff --check`, and
  `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — PASS.

No guest image, Vulkan CTS binary, browser session, native comparison, or performance protocol ran.
