# WebBoxVM — Active Todo

Completed sprint history moved to [sprint-history.md](sprint-history.md). Aspirational backlog moved to [future.md](future.md).

## Sprint 7 — Serial Linux Userspace and ISO Installer
- [ ] Kernel boots to BusyBox `ash` shell
  - Early console works; next target is enough init, scheduler, device, and initrd behavior to spawn `/init`
  - Default initrd contains real static ARM64 BusyBox, `/init`, `/dev/console`, and applet symlinks
- [x] Replace placeholder BusyBox payload with a real static ARM64 BusyBox binary
- [ ] Continue boot beyond early console into initramfs unpacking and `/init`
- [x] Add UART RX path wiring for interactive shell input
- [x] Add first ARM64 ISO terminal boot path by extracting kernel/initrd from ISO9660 media
- [x] Attach booted ISO media as a read-only VirtIO block device
- [x] Add second writable sparse VirtIO disk for installer target storage
- [x] Add sparse install-disk snapshot/restore format
- [x] Debian ARM64 netinst reaches the serial text installer language prompt
- [x] Add sparse physical memory so browser builds do not allocate the full guest address layout up front
- [x] Validate standard Debian ARM64 netinst native boot through `/lib/debian-installer/menu` and `/usr/bin/main-menu`
- [ ] Standard boot for `CONFIG_RELOCATABLE=n` kernels
  - [ ] Add kernel `PAGE_OFFSET` to TTBR1 identity mapping
  - [ ] Map kernel VA range to physical load address before EFI stub runs
  - [ ] Make EFI stub `_text == *image_addr` checks succeed without relocation
  - [ ] Boot kernels at linked VA with MMU already active
  - [ ] Support pre-built Debian/Ubuntu kernels without Docker rebuild
- [ ] Interactive commands: `ls`, `echo hello`, `cat /proc/cpuinfo`

**Result so far:** Native CLI Debian ARM64 netinst reaches the real text installer language prompt. Remaining work is browser delivery, input responsiveness, and shell/install interaction quality.

## Sprint 8 — Browser Terminal Delivery
- [x] Build `wasm32-unknown-unknown` package with `wasm-bindgen`
- [x] Add browser app shell in `web/`
- [x] Render xterm.js serial terminal
- [x] Add ISO picker and Debian boot button
- [x] Wire browser keyboard input to the guest PL011 UART receive path
- [x] Add pause, resume, reset, step-slice, and live VM metrics
- [x] Add `make web`, `make web-pkg`, and `make web-debian-arm64`
- [x] Verify the browser page loads and terminal DOM renders
- [x] Expose browser install-disk size control and sparse disk allocation metric
- [x] Persist browser install disk through OPFS snapshots with autosave and manual save/clear controls
- [ ] Run Debian ARM64 netinst to the installer language prompt inside the browser app
- [ ] Verify interactive browser input at the Debian prompt
- [ ] Improve browser runtime speed enough for practical terminal interaction
- [ ] Keep generated `web/pkg/` reproducible and uncommitted
