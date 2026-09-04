# VirGL2 capset boundary

## Question

What standards-facing step can WebBoxVM take toward guest graphics compatibility
without advertising Mesa, OpenGL, Venus, or Vulkan behavior it does not provide?

## Finding

VirGL2 is VirtIO-GPU capset ID 2.  It is an evolution of the VirGL renderer
capability layout, not Venus.  Upstream virglrenderer sends both capset 1 and
capset 2 contexts through its GL renderer path; Venus has its own capset ID 4
and a different Vulkan renderer path.

The current upstream `virgl_caps_v2` is a growable 1,376-byte structure.  Its
first 308 bytes are the `virgl_caps_v1` prefix.  The capset-2 maximum version
is 2; versions 0, 1, and 2 use that same growable shape upstream.

## WebBoxVM contract

WebBoxVM now reports capset index 1 as ID 2, version 2, size 1,376 and accepts
ID-2 contexts, resource attachment, detach, and the same bounded standard
VirGL stream accepted for capset 1.  The first 308 bytes repeat the existing
truthful conservative format, primitive, and UBO declarations.  Every expanded
field is zero until its corresponding generic operation has an implementation
and an end-to-end guest test.

This is intentionally useful but narrow.  A guest can negotiate and preserve
the actual standard capset-2 envelope, while it cannot infer unsupported
compute, shader, image, video, multisample, or external-memory features from
made-up capability bits.  It is not a claim that a Mesa VirGL screen can
initialize successfully against the bounded command parser.

## Why Venus is deferred

The VirtIO specification assigns Venus capset ID 4.  Mesa describes Venus as
a serialized Vulkan protocol, and QEMU documents its dependence on host blob
memory.  Browser WebGPU has no Linux dma-buf/file-descriptor external-memory
equivalent, so the existing host-visible byte buffers cannot truthfully stand
in for Venus object memory or synchronization.

The next Venus work therefore remains resource residency, transfer ownership,
and synchronization design, followed by generated protocol support and a
capset whose properties are backed by those layers.

## Evidence and validation

The Rust tests query both capset-info and capset data, assert the exact
1,376-byte response and zero extension tail, then create a capset-2 context,
attach a standard texture resource, and submit an ordinary VirGL clear.  This
proves capset discovery and routing; it does not prove general GL.

Sources:

- [OASIS VirtIO GPU capsets](https://github.com/oasis-tcs/virtio-spec/blob/master/device-types/gpu/description.tex)
- [VirGL capability layouts](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/virgl_hw.h)
- [VirGL renderer capset/context routing](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/virglrenderer.c)
- [VirGL renderer capability versions](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/main/src/vrend_renderer.c)
- [Mesa Venus architecture](https://docs.mesa3d.org/drivers/venus.html)
- [QEMU VirtIO-GPU Venus requirements](https://qemu.googlesource.com/qemu/+/master/docs/system/devices/virtio/virtio-gpu.rst)
