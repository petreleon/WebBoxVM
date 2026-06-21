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
- Debian reaches base install `73%`, then `localedef` sleeps in
  `balance_dirty_pages`.
- Guest `/proc/meminfo` is corrupt: plausible 32-bit counters appear shifted
  into high 32 bits.
- Suspect ARM64/JIT register-width or wide-immediate bug; fix, test, restart.

## Done, Compressed
- [x] Modular emulator architecture drafted with clear subsystem boundaries.
- [x] Source-file size limit enforced by test: max 180 lines.
- [x] Browser ISO boot writes persistent OPFS disk state.
- [x] WebSocket hub plus Linux Docker NAT peer routes installer traffic.
- [x] Debian installer sees VirtIO-net; DHCP, DNS, and mirror fetch work.
- [x] Debian installer sees, partitions, and writes VirtIO disk `vdb`.
- [x] Ext4 installer module hook loads required crypto/checksum deps.
