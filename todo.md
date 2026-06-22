# WebBoxVM - Active Todo

History: [sprint-history.md](sprint-history.md).

## Now
- [ ] Investigate the installed Debian systemd/BPF/ftrace hang before serial login.
- [ ] Continue isolating the ARM64/JIT semantics bug after installer proof.
- [ ] Keep `Boot ISO` only for installer/media boot.

## Learn / Debug Queue
- [ ] Explain `web/js/vm-worker/jit-compile.js` JIT policy
- [ ] Trace opcode telemetry for `0x6e20ac00` / `Opcode::SimdUminp`

## Current Blocker
- `Boot disk` reaches installed Debian root fsck/mount and systemd.
- Serial login is blocked by a later hung-task path around BPF/ftrace teardown.
- `?bootargs=sysctl.kernel.ftrace_enabled=0` still reproduces the same BPF/ftrace hang.

## Done
- [x] Guardrails, OPFS `Boot disk`, browser NAT/DHCP/DNS/HTTP, installer reached reboot.
- [x] Final browser disk snapshot compacted, persisted in OPFS, and booted via `Boot disk`.
- [x] Added opt-in installed-disk bootargs for faster browser blocker probes.
