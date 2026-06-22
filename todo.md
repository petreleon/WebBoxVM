# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Verify `Boot disk` against a real saved Debian install snapshot.
- [ ] Finish the fresh Debian install and save the final browser OPFS disk snapshot.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- A fresh installer run is recreating the browser OPFS disk snapshot.
- Final proof remains: installed system boots from the writable browser disk via the `Boot disk` path.

## Done
- [x] Modular architecture guardrails, 180-line source limit, tests, OPFS `Boot disk`, browser NAT/DHCP/DNS/HTTP, and one installer finish-install path over NAT.
