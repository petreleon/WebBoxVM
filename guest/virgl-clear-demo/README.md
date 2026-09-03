# WebBoxVM standard VirGL clear probe

This freestanding AArch64 Linux program is a deliberately small guest-side
proof for the standard capset-1 clear path. It is not Mesa, OpenGL, or Vulkan.

It opens `/dev/dri/card0`, reads the Linux `virtgpu` capset-1 response, creates
a B8G8R8A8 render-target resource, views it through KMS's XRGB primary-plane
format at the 1024×768 scanout, submits standard VirGL `OBJECT_SURFACE`,
`SET_FRAMEBUFFER_STATE`, and generic `CLEAR` commands, then waits for the
resource fence before closing the DRM file.

The wait matters: WebBoxVM completes the guest submission only after the
browser WebGPU queue reports completion. Closing the context before that point
would invalidate the pending standard-VirGL effect.

## Build

```sh
make -C guest/virgl-clear-demo
```

The default cross toolchain is `aarch64-elf-gcc`; set `CROSS` for another
freestanding AArch64 GNU toolchain. The verifier checks ELF linkage, ABI-sized
ioctl structures, no undefined symbols, the 64 KiB artifact cap, and the
180-line maintained-file limit.

## Guest result

Inject the built program into an installed WebBoxVM Debian guest after loading
`virtio_gpu`, then run it as the DRM master on the serial console. Success is:

```text
VIRGL_CLEAR_DEMO_PASS card0 capset=1 clear=64,128,191,255
```

That marker appears only after the guest fence resolves. A validation harness
must also inspect the emitted `VGC1` and final `WBGF` packets to establish the
browser completion and BGRA readback, respectively.

The fixed dimensions and one color target are intentional. A mode, format,
KMS, resource, or command mismatch fails at the named stage instead of being
silently treated as general graphics compatibility.
