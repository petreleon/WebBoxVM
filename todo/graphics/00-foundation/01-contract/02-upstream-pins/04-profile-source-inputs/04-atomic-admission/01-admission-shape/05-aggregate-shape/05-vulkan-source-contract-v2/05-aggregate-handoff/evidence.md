# F02.4.4.1.5.5.5 evidence

Revision: `dfb0a236c3e170fdb668db7b28634c0069757417`.
Validation: handoff 8/8; checker unit 10/10; external-cache replay 98 members; `make test` passed.
Result: PASS
Artifacts: handoff `8b286471…e682ee`; cache receipt `56fd45c1…b8437`; live receipt `48c4d54d…481599`.
Profile: V2 receipt handoff only; no CTS execution, guest/browser route, compatibility, conformance,
certification, performance, inventory cutover, or F03 transition.

## Immutable handoff

The self-hashed [handoff](vcts_aggregate_handoff.json) is
`8b286471e95a957c7d1c9fc9cf41c26d246de86ad1a7cbffb4fb58c3fde682ee`. It binds:

- root identity `30b272f8c563e0dabf307795c01496eb70f744514b4790439ebbfc69c7bd5218`;
- tree plan `77875b165876a3cf37450667c7e9c23e177bff8a7795e0d48b4214e202064804`;
- ordered ledger `608d520463bfd4724e79b52ee639a12d446e14c5e8d7b3687872eaf3c4e917f0`,
  98 members and 434,669,348 B;
- cache receipt `56fd45c16f1f1e1c68bfc76f3853215eaff08c71ec60e5ceb85c5ecb26ef8437`;
- live capture receipt `48c4d54d8738a8eab526347bc8080c245bb9cef269b66011e32116f78f481599`;
- taxonomy `752a3ae5ff10ea0d9f9a2638c0d1612fb7af3d376dabad1601f8b9c1e76cb2f7`.

The checked cache fixture is a small semantic copy of the external receipt, not CTS payload. Its source
receipt bytes had SHA-256 `41abf2e502fdea0ca6961359b1c10259a686b9056a0d84e13b4165bc246bcf8b`;
the committed fixture validates its canonical receipt self-hash above. The ~415 MiB payload remains external.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`, Python 3.14.6 ran:

```text
V2_HANDOFF=todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/05-aggregate-handoff
V2_LEDGER=todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/02-live-closure/vcts_closure_ledger.json
python3 -B "$V2_HANDOFF/vcts_aggregate_handoff.py"
python3 -B "$V2_HANDOFF/vcts_aggregate_handoff_test.py"
python3 -B todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/04-atomic-admission/01-admission-shape/05-aggregate-shape/05-vulkan-source-contract-v2/03-external-closure-cache/01-cache-contract/vcts_cache_cli.py --verify --cache-root /private/tmp/webboxvm-vcts-v2-live-20260910-r2 --ledger "$V2_LEDGER"
python3 -B scripts/test_check_graphics_roadmap.py
python3 -B scripts/check_graphics_roadmap.py
make test
git diff --check
```

The adapter returned `HANDOFF: 98 members ... unadmitted`; its 8 positive/hostile tests passed with
0 failures. The cache replay returned `PASS: reused 98 members 434669348 bytes` with the pinned ledger.
Checker tests passed 10/10 and the roadmap checker reported 299 documents, 178 tasks, 52 PASS-complete,
and 27 superseded before this leaf was closed. `make test` exited 0: Rust 1,127/0/3, source limits 6/0/0,
and Node 337/0/0.

The hostile cases reject a V1 audit or stale root, partial/reordered/mixed ledger, a re-sealed bad cache
receipt, incomplete or admitting live receipt, reclassified/detached taxonomy, injected inventory field,
admitted Docs value, tampered hash, duplicate keys, and an oversize handoff before JSON decoding.

## Boundary and decision

All three state bits remain false: `admitted`, `cutover_ready`, and
`satisfies_vulkan_14_core_manifest`. `vk-default.txt` is a Khronos-hosted SHA-pinned selector broader than
Vulkan 1.4 core; local filtering is forbidden. The taxonomy records 0 core, 1 WSI, 1 video, 4 extension,
and 93 unknown members. Historical generated Vulkan Docs are provenance only, outside the admitted
implementation-source closure, and are neither imported nor resolved here. Tag-signature verification was
not performed.

The aggregate admission proof may consume this receipt later, but this handoff does not prove that proof,
modify V1 evidence, or alter inventory/F03 state. Commit `dfb0a236` was pushed to
`codex/graphics-f01-baseline`; the next task is F02.4.4.1.5.4, aggregate closure admission.
