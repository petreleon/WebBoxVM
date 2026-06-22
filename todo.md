# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Optimize default installed-disk boot speed after serial login proof.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Add or intentionally skip JIT support for observed `Stxp` hot block `0xc8270c82`
- [x] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- No disk-boot correctness blocker observed: default browser `Boot disk` reaches Debian 13 `debian login:` on `ttyAMA0`.
- Main work now: speed; current login proof is about 642 seconds of browser wall time.

## Recent Proofs
- Browser install path works: NAT/DHCP/DNS/HTTP, installer reboot, compacted final disk snapshot, OPFS `Boot disk`.
- Systemd hang was isolated after root handoff in the BPF/ftrace path; `init=/bin/sh` worked, ftrace/emergency probes still hung.
- Current default avoids that path, masks slow/unneeded services, uses serial-only getty, batches UART, and reaches login in about 642s.
- JIT-enabled disk probe no longer rejects observed `MRS DCZID_EL0` (`0xd53b00e3`); next reject is `Stxp` (`0xc8270c82`).

## Done
- [x] Browser installer networking and OPFS disk persistence.
- [x] Default persisted-disk boot to Debian 13 serial login.
- [x] Bootarg probes, BPF/LSM workaround, service masks, serial-only getty, and UART batching.
- [x] JIT skip/reject/fallback stats include current instruction snapshots.
- [x] JIT can compile observed side-effect-free `MRS DCZID_EL0` and `TPIDRRO_EL0` reads.
