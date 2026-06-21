# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Add and verify `Boot from disk`.
- [ ] Continue installer from the `20260621-ldrsb-fix` browser build.
- [ ] Verify mirror/package fetch and base install after the VM-stat fix.
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
- Fresh browser proof after `20260621-ldrsb-fix`: VirtIO-net appears as `eth0`,
  DHCP completes, and `partman` formats the writable VirtIO disk.
- Next installer proof: mirror/package fetch and base install complete.
- Final proof: installed system boots from the writable disk.

## Done
- [x] Modular emulator architecture with 180-line source guard.
- [x] Browser disk persistence, ext4 install path, and OPFS quota handling.
- [x] WebSocket hub plus Docker NAT prove installer VirtIO-net/DHCP.
- [x] VirtIO disk partitioning/formatting reaches base install in browser.
- [x] ARM64 signed-load bug fixed; VM stats verified clean.
