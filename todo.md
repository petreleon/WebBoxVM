# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Optimize disk boot speed.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Speed only: `Boot disk` reaches Debian 13 login in 490.3s latest / 486.1s best; latest JIT has 0 rejects, 0 skips, 2 timer-deadline fallbacks.

## Done
- [x] NAT + Debian disk boot works: installer path proven, login reached, `Boot ISO` split, ESR_EL1 + LDXP + SP_EL0 MSR JITed, 594.3s -> 486.1s best.
