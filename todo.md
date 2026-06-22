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
- Speed: latest browser disk-boot proof reached `debian login:` at 594.3s with 0 JIT rejects/skips and one safe timer-deadline fallback.

## Done - Compressed
- [x] Install/network/storage: browser NAT/DHCP/DNS/HTTP install, reboot, compact persistent OPFS disk.
- [x] Boot/defaults: `Boot disk` is default; `Boot ISO` is media-only; Debian 13 reaches `ttyAMA0` login.
- [x] JIT/proof/speed: saved-disk JIT stats/rollback; EL0/sysreg/`SPSR_EL1`/`LDAR`/timer tests; 32-batch pump plus fused prepare/finish, preflight, streaming/page-gen/endpoint validation samples reach ~2.56-2.74B steps in ~150-155s with 0 rejects/skips.
