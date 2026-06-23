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
- Speed only: `Boot disk` reaches Debian 13 `ttyAMA0` login in 520.9s latest / 520s best post-click-runner time, improved from 594.3s. Latest JIT stats: 0 rejects, 0 skips, 0 fallbacks; deferred background autosave keeps the saved-disk UI stable during boot instead of snapshotting mid-run.

## Done
- [x] Browser NAT + Debian disk boot reaches login.
