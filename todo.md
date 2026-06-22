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

## Done - Extremely Compressed
- [x] Browser Debian install works end-to-end via NAT/DHCP/DNS/HTTP, reboots from persistent compact OPFS disk by default, reaches Debian 13 `ttyAMA0` login, and has sync-JIT/stats/probe-cache/pump/runtime-progress/timestamp/metrics/uart-poll/network-tx/autosave speed proofs at ~2.56-2.74B steps in ~150-155s with 0 rejects/skips.
