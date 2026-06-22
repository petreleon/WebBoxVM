# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [x] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Optimize default installed-disk boot speed after serial login proof.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- No installed-disk correctness blocker observed: default browser `Boot disk` reaches `debian login:` on `ttyAMA0`.
- Remaining concern: boot speed; current login proof is about 660 seconds of browser wall time.

## Recent Proofs
- Plain installed-disk systemd boot was blocked by a hung-task path around BPF/ftrace teardown.
- `?bootargs=sysctl.kernel.ftrace_enabled=0` still reproduces the same BPF/ftrace hang.
- `?bootargs=systemd.unit=emergency.target` still reproduces the same BPF/ftrace hang.
- `?bootargs=init=/bin/sh` reaches a shell and prints `/proc/cmdline` plus `uname`.
- The old blocker was isolated after initramfs/root handoff, inside systemd's BPF/ftrace path.
- Default installed-disk boot omits the BPF LSM and masks keyboard/console setup to reach serial login.
- Masking AppArmor reduced login proof time from about 800 seconds to about 660 seconds.
- Default browser `Boot disk` with the AppArmor mask reaches serial login in about 661 seconds.

## Done
- [x] Guardrails, OPFS `Boot disk`, browser NAT/DHCP/DNS/HTTP, installer reached reboot.
- [x] Final browser disk snapshot compacted, persisted in OPFS, and booted via `Boot disk`.
- [x] Added opt-in installed-disk bootargs for faster browser blocker probes.
- [x] Default browser `Boot disk` reaches Debian 13 serial login from persisted disk.
- [x] Masked AppArmor for default installed-disk boot to reduce time to serial login.
