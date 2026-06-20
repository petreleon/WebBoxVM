# WebBoxVM — Sprint History

## Done — Extremely Compressed
- CPU/emulator foundation: ARM64 state, decode/execute, RAM/MMIO, PL011, tests.
- Linux boot path: EFI stubs, PE relocations, decompressor entry, MMU/TLB, timer/IRQ/exception basics.
- Userspace/install media: cpio initrd, BusyBox shell, ISO9660 Debian netinst extraction, VirtIO block ISO plus writable sparse disk.
- Browser runtime: wasm64 Memory64 build, Worker execution, xterm terminal, metrics, OPFS sparse disk persistence.
- Installer progress: Debian netinst reaches serial UI, loads ISO components, exposes disk/network devices, proves DHCP/DNS/package traffic through WebSocket NAT.

## Current Edge
- Finish Debian install from the next real blocker, then add `Boot from disk` from the persisted OPFS disk.
