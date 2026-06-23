# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Debian login hang investigated.
- [ ] Optimize default disk boot speed.
- [ ] Isolate ARM64/JIT semantics bug.
- [x] `Boot ISO` = installer/media only.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Speed only: `Boot disk` reaches Debian 13 `ttyAMA0` login in 519.5s latest / best post-click-runner time, improved from 594.3s. Latest JIT stats with 2.5M probe slices: 1068 cache blocks, 1 unsupported-MRS reject, 0 skips, 8 timer-deadline fallbacks.

## Done
- [x] Browser NAT + Debian disk boot reaches login.
