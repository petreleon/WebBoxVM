# WebBoxVM Runtime Assets

The local `.artifacts/busybox-aarch64` file is the ARM64 BusyBox binary embedded into the default initrd.

Current source:

- Docker image: `busybox:1.37.0-musl`
- Platform: `linux/arm64`
- Binary path: `/bin/busybox`
- Format: static-pie ELF64 AArch64

Regenerate it with:

```bash
scripts/update_busybox.sh
```

`.artifacts/` is intentionally ignored because it also contains local kernel images and other large runtime blobs.
