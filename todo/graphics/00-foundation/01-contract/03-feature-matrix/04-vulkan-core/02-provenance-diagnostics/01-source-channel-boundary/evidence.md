# F03.4.2.1 evidence — sealed Vulkan source channels

Revision: `65832c38560e8e1f65d6b26fac1f811f77b623d2`
Validation: `make graphics-profile-source-gate-test graphics-vulkan-registry-inventory-test graphics-vulkan-source-channel-boundary-test`; `make graphics-f05-source-adapter-test graphics-profile-source-gate-test graphics-vulkan-source-channel-boundary-test`; `cargo test -p emulator --test source_file_limits --quiet`; `make test`; `git diff --check`; `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py`
Result: PASS
Artifacts: `vulkan_source_channels.json` SHA-256 `f90b96e2f90e7fc72816c6112c783bbf90069c99d063f8d9884ccaa36598c847`; embedded boundary self-hash `dd07b39283a3e41edf217e20dbb1eec90e8e6e73b6aae748d46bb57588983683`
Profile: Vulkan 1.4 core provenance boundary only; `blocked` / `matrix-incomplete`; no Docs closure, matrix row, or CTS execution

Task ID and date: F03.4.2.1, 2026-09-21.

Status: PASS for the source-channel boundary only; Vulkan 1.4 remains `blocked` by
`matrix-incomplete`.

## Receipt

[`vulkan_source_channels.json`](vulkan_source_channels.json) is self-hashed as
`dd07b39283a3e41edf217e20dbb1eec90e8e6e73b6aae748d46bb57588983683`. It binds the active
role-aware contract and lock, records the exact `vulkan-14-spec` and `vulkan-cts-default` root
identities with revision, digest, bytes, license, and attribution, and records zero CTS executions,
zero matrix rows, and all WebBoxVM claims false.

The Docs root permits only a separately pinned citation map. `vkspec.adoc` includes are not a Docs
closure. The unfiltered `vk-default.txt` root is diagnostic-only. F03.4.1's `vulkan-registry`
identity is revalidated through its fixed v2 scaffold as auxiliary structural input: its row policy
requires `blocked`, null owner, null independent test plan, and no matrix ingress.

## Verification

- `make graphics-profile-source-gate-test graphics-vulkan-registry-inventory-test graphics-vulkan-source-channel-boundary-test` — PASS: F03 source gate 24 tests, registry v2 11 tests, boundary 6 tests.
- `make graphics-f05-source-adapter-test graphics-profile-source-gate-test graphics-vulkan-source-channel-boundary-test` — PASS after the F05 schema probe stopped fabricating a pre-citation Vulkan row.
- `cargo test -p emulator --test source_file_limits --quiet` — PASS: 6 tests.
- `make test` — PASS: graphics targets, 1,127 Rust unit tests (3 ignored), source-file limits, and 338 Node tests.
- `git diff --check` and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py` — PASS before this evidence update.

The v2 matrix hook now rejects every Vulkan row until F03.4.2.2 admits a citation map; it gives a
specific rejection for `xml/vk.xml` registry locators. This is a provenance guard, not a Docs locator,
CTS selection or execution, guest/browser result, conformance/certification claim, or performance
measurement.
