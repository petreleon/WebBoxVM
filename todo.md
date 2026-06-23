# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Optimize disk boot speed.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy

## Current Blocker
- Speed only: `Boot disk` reaches Debian 13 login in 490.3s latest / 486.1s best; latest JIT has 0 rejects, 0 skips, 2 timer-deadline fallbacks.
- Measured non-wins not kept: metrics 500ms 500.9s, frame 48ms 497.5s, JIT hot threshold 1 511.4s, VirtIO block scratch buffer 504.1s, step slice 10M 505.9s, JIT block cap 96 518.0s.

## Done
- [x] NAT + disk boot proven: Debian installer/login works; best 486.1s.
