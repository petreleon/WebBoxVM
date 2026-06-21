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
- Fresh interpreter-only boot reaches partitioning, DHCP succeeds after retry,
  and `vdb` is partitioned.
- Live run used stale wasm without `05webboxvm_ext4`, so ext4 mount failed.
- Local `web/pkg` is rebuilt and contains the ext4 hook; rerun clean install.
- Continue isolating the exact ARM64/JIT semantics bug after installer proof.

## Done, Compressed
- [x] Modular emulator architecture drafted with clear subsystem boundaries.
- [x] Source-file size limit enforced by test: max 180 lines.
- [x] Browser ISO boot writes persistent OPFS disk state.
- [x] WebSocket hub plus Linux Docker NAT peer routes installer traffic.
- [x] Debian installer sees VirtIO-net; DHCP, DNS, and mirror fetch work.
- [x] Debian installer sees, partitions, and writes VirtIO disk `vdb`.
- [x] Ext4 installer module hook loads required crypto/checksum deps.
