# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Optimize disk boot speed.
- [ ] Fix next ARM64/JIT blocker: `MSR` raw `0xd518411c`.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Speed only: `Boot disk` reaches Debian 13 login in 486.1s; next JIT reject is `MSR` raw `0xd518411c`.

## Done
- [x] NAT + Debian disk boot works: installer path proven, login reached, `Boot ISO` split, ESR_EL1 + LDXP JITed, 594.3s -> 486.1s.
