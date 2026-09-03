# WebBoxVM experimental WebGPU guest demo

This is a freestanding AArch64 Linux program for WebBoxVM's experimental
`WBG3` context. It uses raw syscalls, has no libc or ELF interpreter, selects
private capset ID 7, and submits one indexed cube through the standard Linux
virtio-gpu DRM ioctl transport. The command stream is private to WebBoxVM; it
is not VirGL, Venus, Mesa, OpenGL, Vulkan, or a standardized VirtIO capset.

## Build and verify

```sh
make -C guest/webgpu-demo
```

The default cross toolchain is `aarch64-elf-gcc`. Override `CROSS` if the same
freestanding GNU tools use another prefix. Verification checks ELF class,
machine, static linkage, undefined symbols, entry point, file size, maintained
source line counts, and the exact 408-byte WBG3 packet.

## Inject over the existing serial console

Boot the installed Debian fixture, log in as root, and load the driver:

```sh
modprobe virtio_gpu
while [ ! -e /dev/dri/renderD128 ] && [ ! -e /dev/dri/card0 ]; do sleep 1; done
```

On the host, encode the verified binary:

```sh
base64 < guest/webgpu-demo/build/webgpu-demo
```

In the guest console, paste that output between these markers:

```sh
base64 -d > /tmp/webgpu-demo <<'WEBGPU_DEMO_EOF'
PASTE_HOST_BASE64_HERE
WEBGPU_DEMO_EOF
chmod 0755 /tmp/webgpu-demo
/tmp/webgpu-demo
```

Success is exactly one of:

```text
WEBGPU_DEMO_PASS renderD128 capset=7 cube=8/36
WEBGPU_DEMO_PASS card0 capset=7 cube=8/36
```

The program first tries the render node, then the primary card. Failures name
the furthest completed stage: device open, context initialization, or command
submission. `EXECBUFFER` deliberately requests no output fence or sync objects
and uses no BO handles. Its ioctl and PASS marker are therefore asynchronous:
PASS proves submission through the Linux DRM path, but may precede the browser
acknowledgment. A positive browser acknowledgment is sent only after
`GPUQueue.onSubmittedWorkDone()`; draw telemetry plus a pixel check is still
required to prove WebGPU execution and visible output.

A successfully completed WBG3 v1 draw owns exclusive full-canvas presentation.
Later WBGF frames update the 2D shadow but cannot overwrite it; reset, WebGPU
device loss, or a later failed draw returns to 2D. WBG3 v1 has no release opcode.

For a transport-only installed-disk check, run
`scripts/gpu_guest_transport_smoke.sh`. The confirmed native run observed both
DRM nodes, seven full WBGF frames, the exact WBG3 packet and completion, and no
`0x1205` response. That smoke supplies a native host success acknowledgment; it
does not exercise the browser WebGPU queue or prove browser pixels.
