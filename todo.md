# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Restart from the `20260621-tlb-guard` build and re-run Debian base install.
- [ ] Verify `localedef` completes without corrupt `Dirty` / `Writeback` VM stats.
- [ ] Add and verify `Boot from disk`.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Compressed-disk rerun passed the old 47% OPFS quota failure point.
- Base install reached `localedef`, then Linux slept in `balance_dirty_pages`.
- Guest `MemTotal` stayed sane, but `MemFree`, `Dirty`, `Writeback`, and `Cached`
  became impossible VM-stat values.
- Added a guarded TLB cache keyed by translation context and page-table descriptor
  generation; next proof is a clean rerun through `localedef`.
- Next installer proof: base install completes and reaches package-manager/mirror setup.
- Final proof: installed system boots from the writable disk.

## Done
- [x] Architecture modularized; 180-line source guard added.
- [x] Browser ISO boot persists disk state in OPFS.
- [x] WebSocket hub + Docker NAT route installer networking end to end.
- [x] Debian installer reaches disk partitioning with VirtIO-net/disk working.
- [x] Ext4 hook reaches fresh browser guest; parent cpio dirs fixed.
- [x] Ext4 loads in the installer; `/target` and `/target/boot` mount as ext4.
- [x] Browser disk persistence no longer treats autosave quota as fatal.
- [x] Base install passed 47% on compressed OPFS storage without quota failure.
- [x] Diagnosed the post-73% installer stall as corrupted Linux VM dirty/writeback stats.
