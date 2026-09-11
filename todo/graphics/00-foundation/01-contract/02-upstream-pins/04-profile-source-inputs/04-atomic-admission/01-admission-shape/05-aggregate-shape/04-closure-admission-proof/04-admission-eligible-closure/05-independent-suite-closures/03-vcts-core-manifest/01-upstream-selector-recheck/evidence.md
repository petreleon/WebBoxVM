# F02.4.4.1.5.4.4.5.3.1 — VCTS upstream-selector recheck

Result: **BLOCKED** — `missing-khronos-published-immutable-explicit-vulkan-1.4-core-vcts-manifest`

This is a successful verification of the blocker, not admission, Vulkan CTS execution, certification,
guest API support, browser behavior, or a performance result. The task remains unchecked because the
authority it requires is external and has not been published.

## Live observation — 2026-09-11

The official [release overview](https://github.com/KhronosGroup/VK-GL-CTS/wiki/Overview-of-releases)
and [release page](https://github.com/KhronosGroup/VK-GL-CTS/releases/tag/vulkan-cts-1.4.6.2) identify
`vulkan-cts-1.4.6.2` as the observed latest 1.4.6 family tag. A read-only tag probe recorded:

```text
42c723aa10d2652590f02741827aef43b0421d23 refs/tags/vulkan-cts-1.4.6.2
f6a29701220f34dd1407513bfe80d74ca7b392ce refs/tags/vulkan-cts-1.4.6.2^{}
```

At that immutable peeled commit, the
[mustpass-main directory](https://api.github.com/repos/KhronosGroup/VK-GL-CTS/contents/external/vulkancts/mustpass/main?ref=f6a29701220f34dd1407513bfe80d74ca7b392ce)
contains exactly these root candidates:

```text
vk-default.txt
vk-fraction-mandatory-tests.txt
vksc-default.txt
```

It has no Khronos-published `vulkan-1.4-core` manifest. The pinned
[`vk-default.txt`](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/f6a29701220f34dd1407513bfe80d74ca7b392ce/external/vulkancts/mustpass/main/vk-default.txt)
is 3,347 bytes with SHA-256
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`; it explicitly references both
`vk-default/video.txt` and `vk-default/wsi.txt`. It is therefore the broader canonical diagnostic, not
evidence of a Vulkan 1.4 core selection.

## Sealed offline gate

Commit `b4c8822a` adds
[`upstream_selector_recheck.py`](upstream_selector_recheck.py), its self-hashed
[`record`](upstream_selector_recheck.json), and hostile tests. The record is intentionally dated
`observed_on: 2026-09-11`; an offline validator does not claim that this remains current after that
observation.

It binds semantic and raw SHA-256 identities for the V2 root/handoff, successor boundary, source/release
boundary, active F02 inventory lock, and active F03 requirements. It requires all of the following to stay
false: local core selection, taxonomy promotion, root-only evidence, opaque splitting, VCTS-as-Docs,
tag-alone sufficiency, admission/support/conformance effects, and
`satisfies_vulkan_14_core_manifest`.

The historic global blocker remains
`vcts-vk-default-compound-oversize-core-scope-unadmitted`; this receipt does not replace it. The active
boundary remains 98 V2 members / 434,669,348 bytes, with 14 members over the 8 MiB F02 cap and category
counts `core=0`, `wsi=1`, `video=1`, `extension=4`, `unknown=92`.

## Verification

```text
env -u PYTHONPATH PYTHONDONTWRITEBYTECODE=1 python3 -B upstream_selector_recheck_test.py
4 tests passed

python3 -B upstream_selector_recheck.py
BLOCKED: missing-khronos-published-immutable-explicit-vulkan-1.4-core-vcts-manifest
0157a3c6e5015c68307f224aa236d64107d021613b93bb3d8f70c7934fb971ee
```

The tests reject re-sealed observation/scope changes, local promotion, `0` substituted for a strict
boolean false, stale raw anchors, duplicate/non-finite JSON, oversize input, FIFO, and final or
intermediate-directory symlink paths.
The V2 handoff (8 tests), successor boundary (3), and source/release boundary (6) also passed.

## Required next condition

Khronos must publish an immutable, provenance-bearing artifact whose explicit scope is complete Vulkan 1.4
core. A new tag alone, `vk-default`, `vk-fraction-mandatory-tests`, `vksc-default`, a local filter, or a
taxonomy report is insufficient. Then continue with
[F02.4.4.1.5.4.4.5.3.2](../02-validate-authoritative-core-manifest/README.md).
