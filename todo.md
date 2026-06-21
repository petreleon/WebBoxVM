# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Finish the live Debian base install from the browser run.
- [ ] Add and verify `Boot from disk`.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Base install is running from `file:///cdrom/`; last verified screen reached 26%.
- Next proof: base install completes and reaches package-manager/mirror setup.
- Final proof: installed system boots from the writable disk.

## Done
- [x] Architecture modularized; 180-line source guard added.
- [x] Browser ISO boot persists disk state in OPFS.
- [x] WebSocket hub + Docker NAT route installer networking end to end.
- [x] Debian installer reaches disk partitioning with VirtIO-net/disk working.
- [x] Ext4 hook reaches fresh browser guest; parent cpio dirs fixed.
- [x] Ext4 loads in the installer; `/target` and `/target/boot` mount as ext4.
