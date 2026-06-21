# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Fix the ARM64/JIT corruption blocking Debian base install.
- [ ] Resume installer, finish install, then add `Boot from disk`.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Clean cache-busted boot proves `05webboxvm_ext4` exists in the guest.
- Rerun installer through partition write/base install; verify ext4 loads.
- Continue isolating the exact ARM64/JIT semantics bug after installer proof.

## Done
- [x] Architecture modularized; 180-line source guard added.
- [x] Browser ISO boot persists disk state in OPFS.
- [x] WebSocket hub + Docker NAT route installer networking end to end.
- [x] Debian installer reaches disk partitioning with VirtIO-net/disk working.
- [x] Ext4 hook reaches fresh browser guest; parent cpio dirs fixed.
