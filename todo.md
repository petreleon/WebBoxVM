# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Verify `Boot disk` against a real saved Debian install snapshot.
- [ ] Recreate or recover the browser OPFS install snapshot; current Playwright profile has no saved disk bytes.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- Installer reached finish-install and requested reboot, but the saved OPFS snapshot is not present after reloading the browser profile.
- Final proof remains: installed system boots from the writable browser disk via the `Boot disk` path.

## Done
- [x] Added `Boot disk` browser/wasm/runtime path for persisted sparse install snapshots.
- [x] Added read-only sparse snapshot, GPT/MBR, and ext4 boot artifact extraction.
- [x] Debian installer package selection and finish-install advanced over routed NAT.
- [x] Modular architecture guard: 180-line source limit plus tests.
- [x] Persistent browser disk path: OPFS, ext4, writable VirtIO disk.
- [x] Routed network path: WebSocket hub, Docker NAT, DHCP, DNS, HTTP mirror fetch.
- [x] Installer advanced past old stalls: VM stats, base install, kernel/initramfs.
