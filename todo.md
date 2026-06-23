# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Optimize disk boot speed.
- [ ] Fix next ARM64/JIT blocker: `LDXP`.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Speed only: `Boot disk` reaches Debian 13 login in 518.5s; next JIT reject is `LDXP`.

## Done
- [x] NAT + Debian disk boot works: installer path proven, login reached, `Boot ISO` split, ESR_EL1 JITed, 594.3s -> 518.5s.
