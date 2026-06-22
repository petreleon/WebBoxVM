# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Optimize default installed-disk boot speed after serial login proof.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [x] Decide the EL0 guest-memory-helper JIT policy after the `Stxp` proof
- [x] Add or intentionally skip JIT support for observed `Stxp` hot block `0xc8270c82`
- [x] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- No disk-boot correctness blocker observed: default browser `Boot disk` reaches Debian 13 `debian login:` on `ttyAMA0`.
- Main work now: speed; pre-JIT default login proof was about 642 seconds of browser wall time.
- Latest default-JIT login timing reached systemd/userland at 376s and was still booting at 436.7s; harness final-print failed on BigInt.

## Recent Proofs
- Install: browser NAT/DHCP/DNS/HTTP -> Debian install -> reboot -> compact OPFS disk.
- Boot: default `Boot disk` reaches Debian 13 `ttyAMA0` login in about 642s.
- Hang: BPF/ftrace root-handoff path isolated; default avoids it via bootargs/service/getty/UART changes.
- JIT: telemetry plus observed `MRS DCZID_EL0`/`TPIDRRO_EL0`/`Stxp`; 200s EL0 helper probe had 0 rejects, 0 skips.
- Speed: default saved-disk boot now enables JIT; 160s proof had 7 cached blocks, 2061 hot sites, 0 rejects/skips.
- JIT: observed `MRS CurrentEL` (`0xd5384253`) is safe; 200s default proof had 20 cached blocks, 0 rejects/skips.

## Done
- [x] Browser install/network/disk persistence; default disk boot to serial login.
- [x] BPF/ftrace workaround path; service masks, serial getty, UART batching.
- [x] JIT stats snapshots; safe sysreg reads; observed exclusive pair store.
- [x] JIT helper failures clear staged side effects; EL0 guest-helper blocks are runtime-gated, not compile-skipped.
- [x] Default saved-disk boot uses the proven browser JIT path; media/installer boots stay conservative.
- [x] JIT can compile observed side-effect-free `MRS CurrentEL`.
