# Research: bounded VirGL vertex uniform offset

## Question

Can a standard capset-1 uniform binding move a canonical triangle without
claiming generic vertex UBOs, TGSI, OpenGL, Vulkan, or Venus support?

## Applied contract

WebBoxVM recognizes exactly this vertex program:

```text
VERT
DCL IN[0]
DCL CONST[0][0]
DCL OUT[0], POSITION
ADD OUT[0], IN[0], CONST[0][0]
END
```

Command 27 accepts shader type zero, buffer index zero, one attached R8
constant buffer, a four-byte-aligned offset, and exactly 16 bytes. At draw
preparation it snapshots `[dx,dy,z,w]`. `dx` and `dy` must be finite and in
`[-1,1]`; `z` and `w` must be zero.

The renderer first validates every source position and triangle. It adds the
offset only to local copied X/Y components, then validates the resulting
positions and triangles again before generating schema-2 `VGD1`. Consequently,
post-queue resource mutation cannot change the deferred CPU/WebGPU work.

## Native proof

The AArch64 guest creates one 36-byte R8 constant buffer. It sends isolated
command-9 writes for fragment RGBA at byte four and `[-.015625,0,0,0]` at byte
20, reads all 36 bytes back, then binds command-27 stage zero at byte 20 and
stage one at byte four. The host harness checks the transformed schema-2 vertex
bits, completes the queue, and requires source-over BGRA triangle samples of
`147,141,58,255`.

## Deliberate limits

There is no matrix, array, arbitrary instruction, generic vertex constant,
second slot, non-16-byte range, clip-space repair, or fallback transform. This
is one vertical compatibility seam, not a broad shader implementation.

## Sources

- [VirGL protocol command layout](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/virgl_protocol.h)
- [virglrenderer uniform-buffer decoder](https://android.googlesource.com/platform/external/virglrenderer/+/5fbaeb95136674231649045fa197897ad9bd3ff4/src/vrend_decode.c)
