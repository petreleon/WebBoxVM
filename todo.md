# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Add and verify `Boot from disk`.
- [ ] Continue installer from the `20260621-ldrsb-fix` browser build.
- [ ] Verify DHCP, mirror/package fetch, and base install after the VM-stat fix.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Fixed the early VM-stat corruption visible at the first language prompt.
- Root cause was scalar signed-load decode: `ldrsb xN`/`ldrsh xN` were decoded as
  W-register forms, zero-extending values before 64-bit kernel accounting adds.
- Browser proof on `20260621-ldrsb-fix`: language prompt reached, `/proc/vmstat`
  counters and raw `vm_zone_stat` memory are sane.
- Next installer proof: DHCP, mirror/package fetch, and base install complete.
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
- [x] Added raw browser debug reads for VA/PA counter verification.
- [x] Fixed ARM64 scalar signed-load X/W decode and verified clean VM stats.
