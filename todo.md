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
- Speed: latest browser disk-boot proof reached `debian login:` at about 610s with 0 JIT rejects/skips and one safe timer-deadline fallback.

## Done - Tiny
- [x] Install path: browser NAT/DHCP/DNS/HTTP Debian install, reboot, compact OPFS disk persistence.
- [x] Boot/login path: default `Boot disk`; `Boot ISO` media-only; Debian 13 reaches `debian login:` on `ttyAMA0`.
- [x] JIT path: saved-disk JIT, stats, rollback, EL0/sysreg helpers, observed-op coverage, timer-boundary tests.
