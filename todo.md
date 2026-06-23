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
- Speed only: `Boot disk` reaches Debian 13 `ttyAMA0` login in 594.3s.

## Done
- [x] WS NAT + installer + OPFS disk boot -> Debian login.
