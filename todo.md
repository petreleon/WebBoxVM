# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Optimize default installed-disk boot speed after serial login proof.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- No disk-boot correctness blocker observed: default browser `Boot disk` reaches Debian 13 `debian login:` on `ttyAMA0`.
- Main work now: speed; current login proof is about 642 seconds of browser wall time.

## Recent Proofs
- Browser install path works: NAT/DHCP/DNS/HTTP, installer reboot, compacted final disk snapshot, OPFS `Boot disk`.
- Systemd hang was isolated after root handoff in the BPF/ftrace path; `init=/bin/sh` worked, ftrace/emergency probes still hung.
- Current default avoids that path, masks slow/unneeded services, uses serial-only getty, batches UART, and reaches login in about 642s.

## Done
- [x] Browser installer networking and OPFS disk persistence.
- [x] Default persisted-disk boot to Debian 13 serial login.
- [x] Bootarg probes, BPF/LSM workaround, service masks, serial-only getty, and UART batching.
