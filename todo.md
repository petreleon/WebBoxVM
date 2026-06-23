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
- Correctness: none; `Boot disk` reaches Debian 13 `ttyAMA0` login.
- Speed: slow; latest browser proof 594.3s, 0 JIT rejects/skips, 1 timer fallback.

## Done
- [x] WS NAT install -> OPFS disk boot -> Debian serial login.
