# Research: bounded VirGL split vertex input

## Question

How can WebBoxVM accept the standard VirGL vertex-buffer packet shape without
giving the browser a new unvalidated memory or shader surface?

## Findings

`VIRGL_CCMD_SET_VERTEX_BUFFERS` carries exactly three dwords per buffer:
stride, byte offset, and resource handle. The upstream protocol defines its
payload size as `num_buffers * 3`; Mesa's VirGL encoder emits one such triple
for each `pipe_vertex_buffer`. A type-5 vertex-element object independently
selects a source slot, byte offset, and format per attribute.

This means an interleaved position/UV record and separate position plus UV
buffers are equivalent only after resolving each attribute for a vertex index.
The existing capset already advertises format 29 (`R32G32_FLOAT`) and format
31 (`R32G32B32A32_FLOAT`), so accepting a tight format-29 UV source makes the
advertisement truthful instead of introducing a new capability bit.

## Application

WebBoxVM bounds the source array to three slots, matching its accepted fixed
position, RGBA, and UV contracts. It accepts a zero-to-three triple command;
omitted slots are cleared. Each draw validates that every declared element has
an attached source, exact tight stride, aligned offset, expected resource
format, and vertex-buffer bind before copying the attributes into the existing
normalized VGD1 byte layout.

The invariant is: for every output vertex and declared element, exactly one
validated source range is copied, in element order. Therefore later guest
buffer mutation cannot change either the queued CPU effect or browser packet.
For `N <= 1023` vertices and at most three attributes, snapshot work is
`O(3N)` time and `O(N)` normalized packet memory; no browser parser changes.

## Deliberate limits

Only divisor-zero position, position/UV, position/RGBA, and
position/RGBA/UV layouts are accepted. Sparse arbitrary attribute indices,
strided padding, instancing, and arbitrary Gallium vertex formats remain
unsupported. This is still a bounded capset-1 slice, not Mesa/OpenGL support.

## Sources

- [VirGL protocol layout](https://android.googlesource.com/platform/external/virglrenderer/+/refs/heads/aml_net_341810040/src/virgl_protocol.h)
- [Mesa VirGL encoder](https://fossies.org/linux/mesa/src/gallium/drivers/virgl/virgl_encode.c)
- [VirGL capability layouts](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/virgl_hw.h)
