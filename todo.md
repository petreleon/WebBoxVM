# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Optimize default installed-disk boot speed after serial login proof.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [x] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Correctness: no disk-boot blocker observed; default `Boot disk` reaches Debian 13 `debian login:` on `ttyAMA0`.
- Speed: pre-JIT login proof was about 642s; current JIT proof passed DAIF + STXR and next hit `SimdBicImm` (`0x6f079604`) at about 480s.

## Done - Compressed
- [x] Install/network/disk: browser NAT/DHCP/DNS/HTTP install, reboot, compact OPFS persistence.
- [x] Boot path: default boots persisted disk; `Boot ISO` is installer/media-only; DTB exposes only the saved disk for disk boots.
- [x] Login path: BPF/ftrace hang avoided via bootargs, service masks, serial getty, and UART batching.
- [x] JIT base: default saved-disk JIT, stats, helper rollback, runtime-gated EL0 helpers, safe sysreg reads.
- [x] JIT observed ops: `Stxp`, `LDXR`, `Stxr`, `DaifSet/Clr`, and exact timer-boundary commit behavior covered by tests.
