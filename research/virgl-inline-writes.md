# Research: bounded VirGL resource-inline writes

## Question

Can a guest populate the already-supported resource-backed fragment constant
without relying on a separate `TRANSFER_TO_HOST` ioctl?

## Protocol finding

`VIRGL_CCMD_RESOURCE_INLINE_WRITE` is standard command 9. Its payload starts
with resource, level, usage, stride, layer stride, and a `pipe_box`; bytes
follow at word 12, padded to a dword. virglrenderer accepts a payload of at
least 12 words and passes the full padded data length to its transfer path.

## Applied contract

WebBoxVM accepts one isolated command-9 submission (plus no-op commands) for
an attached capset-1 R8 `PIPE_BUFFER` with exactly constant-buffer bind. It
requires resource nonzero; level, usage, stride, layer stride, y, and z zero;
height and depth one; nonempty byte range fully inside the buffer; exactly
`ceil(width / 4)` data words; and zero dword-padding bytes. The command has no
browser packet or deferred effect: after complete stream validation it copies
the raw bytes into the resource snapshot used later by command 27.

This deliberately does not make command 9 a generic texture or vertex upload,
does not accept mixed state/copy/clear/draw streams, and does not establish a
Mesa, OpenGL, Vulkan, or Venus implementation.

## Evidence

`virgl_uniform_draw.rs` proves the command writes the offset-four color used
by command 27 and schema-2 `VGD1`, while wrong bind, range, nonzero padding,
and mixed streams leave the buffer unchanged. The native AArch64 guest submits
the exact 17-word command (11 transfer fields plus five data words), performs
`TRANSFER_FROM_HOST` readback of the zero prefix and four f32 words, then
draws and checks both source-over triangle samples.

## Sources

- [VirGL protocol command and inline-write fields](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/virgl_protocol.h)
- [virglrenderer inline-write decoder](https://android.googlesource.com/platform/external/virglrenderer/+/5fbaeb95136674231649045fa197897ad9bd3ff4/src/vrend_decode.c)
- [VirGL encoder transfer tests](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/aml_con_341810060/tests/test_virgl_transfer.c)
