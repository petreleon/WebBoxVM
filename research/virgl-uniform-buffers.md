# Research: bounded VirGL resource-backed uniform buffers

## Question

Which standard VirGL command can supply the existing `CONST[0][0]` fragment
shader, one canonical vertex offset, or one canonical vertex matrix from a
resource without turning this
renderer into general Gallium, Mesa, OpenGL, Vulkan, or Venus compatibility?

## Protocol finding

`VIRGL_CCMD_SET_UNIFORM_BUFFER` is command 27. Its payload has exactly five
words: shader type, buffer index, byte offset, byte length, and resource handle.
virglrenderer decodes that exact shape before passing the binding to its renderer.

The standard resource bind bit is `VIRGL_RES_BIND_CONSTANT_BUFFER` (`1 << 6`).
WebBoxVM reports the capset-v1 UBO bit and `max_uniform_blocks == 1`, matching
this one binding. Neither the command nor those caps establishes arbitrary
shader or memory compatibility on its own.

## Applied contract

WebBoxVM accepts object zero, vertex shader type 0 or fragment shader type 1,
index zero, and a nonzero R8 `PIPE_BUFFER` resource carrying exactly 16 bytes
for fragment RGBA or a vertex XY offset, or exactly 64 bytes for a canonical
vertex `DP4` matrix, at a four-byte-aligned offset. The resource must be a
capset-1 resource attached to the same context with only the constant-buffer
bind. The `(offset, length, resource) = (0, 0, 0)` form clears the binding.

The fragment binding replaces the limited inline command-12 source for the
canonical fragment program. At draw preparation, WebBoxVM revalidates the
resource, attachment, bind kind, alignment, and range, then snapshots four or
sixteen little-endian f32 words. Fragment values require finite normalized
RGBA; the 16-byte vertex form requires `[dx,dy,0,0]`, finite `dx/dy` in
`[-1,1]`, and adds it to copied positions only after source geometry validates,
then validates again. The 64-byte form is reserved for the exact canonical
four-row `DP4` transform and applies its bounded homogeneous-coordinate rules.

The companion command-9 inline-write path can populate this same attached
buffer in an isolated submission. It copies only a validated byte range after
the entire stream validates, leaving command 27 as the separate binding and
draw contract; `research/virgl-inline-writes.md` records its exact limits.

Invalid wire shapes, stage/index choices, attachment, resource kind, alignment,
or range reject the cloned stream without replacing a prior binding. Detaching a
bound resource clears the source. Nonmatrix material remains schema-2 `VGD1`;
the exact non-depth solid DP4 form may instead use private schema 15 with a
matrix uniform for WebGPU presentation, while CPU replay keeps the same guest
color contract.

## Evidence and limits

`virgl_uniform_draw.rs` proves a nonzero offset reaches the schema-2 color and
source-over BGRA result. It mutates the source buffer after enqueueing to prove
the deferred work retained its snapshot. It also proves malformed, wrong-bind,
unaligned, out-of-range, and missing-resource rejection, standard unbind, and
detach clearing.

The native Linux DRM guest probe creates a 36-byte R8 constant buffer, writes
fragment RGBA at byte four and the vertex vector at byte 20 through command 9,
reads it back, and emits both exact 16-byte command-27 ranges. Its host smoke
accepts schema-2 only with the distinct offset-four RGBA and transformed
vertices and, after deferred completion, requires both source-over BGRA triangle
samples to be `147,141,58,255`.

`virgl_vertex_uniform_draw.rs` proves that the translated schema-2 vertices are
snapshotted before later resource mutation, rejects non-planar offsets, and
proves clear/detach invalidation. The native guest uses two command-9 writes:
fragment RGBA at byte four and a `-0.015625` X offset at byte 20.

`virgl_matrix_uniform_draw.rs` proves that a command-27 64-byte vertex binding
snapshots the canonical `DP4` matrix into a private raw-vertex v15 packet before
later resource mutation, retains transformed CPU replay, and rejects an
unaligned, short, or out-of-range matrix range transactionally.

That establishes this bounded guest-driver transport route alongside Rust and
browser WebGPU-unit tests; it does not establish native guest-to-browser
execution from the harness. There is no
general vertex UBO, nonzero slot, arbitrary range (only exact 16- and 64-byte
forms), uniform array, arbitrary TGSI, generic UBO schema, external memory,
synchronization protocol, or Venus capset claim.

## Sources

- [VirGL protocol command layout](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/virgl_protocol.h)
- [virglrenderer uniform-buffer decoder](https://android.googlesource.com/platform/external/virglrenderer/+/5fbaeb95136674231649045fa197897ad9bd3ff4/src/vrend_decode.c)
- [VirGL capset and resource-bind layout](https://android.googlesource.com/platform/external/virglrenderer/+/056b3873e41c015249499dbf9f761c8e9a78b720/src/virgl_hw.h)
- [virglrenderer resource bind flags](https://android.googlesource.com/platform/external/virglrenderer/+/refs/tags/android-14.0.0_r25/src/virglrenderer.h)
- [Mesa VirGL encoder](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/src/gallium/drivers/virgl/virgl_encode.c)
