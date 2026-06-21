# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Add and verify `Boot from disk`.
- [ ] Continue installer from the `20260621-ldrsb-fix` browser build.
- [ ] Verify mirror/package fetch and base-install completion.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- At base install 98%, `console-setup.postinst` is waiting on long-running
  `ckbcomp` via `setupcon --save-only`.
- Next proof: package-manager work starts.
- Final proof: installed system boots from the writable disk.

## Done
- [x] Architecture guard: modular emulator layout plus 180-line source limit.
- [x] Browser install path: persistent OPFS disk, ext4, and writable VirtIO disk.
- [x] Network path: WebSocket hub plus Docker NAT; installer sees `eth0` and DHCP.
- [x] ARM64 signed-load fix removed early VM-stat corruption.
- [x] Base install passed the old 73% stall, installed kernel/initramfs, and hit 98%.
