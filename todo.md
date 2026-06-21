# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Add and verify `Boot from disk`.
- [ ] Continue installer from the `20260621-ldrsb-fix` browser build.
- [ ] Finish package selection/install from routed NAT.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Installer reached pkgsel; current cleanup is removing/retrying `cdrom` apt media.
- Final proof: installed system boots from writable browser disk.

## Done
- [x] Modular architecture guard: 180-line source limit plus tests.
- [x] Persistent browser disk path: OPFS, ext4, writable VirtIO disk.
- [x] Routed network path: WebSocket hub, Docker NAT, DHCP, DNS, HTTP mirror fetch.
- [x] Installer advanced past old stalls: VM stats, base install, kernel/initramfs.
