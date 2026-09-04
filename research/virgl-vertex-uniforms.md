# Research: bounded VirGL vertex uniform offset

## Question

Can a standard capset-1 uniform binding move a canonical triangle without
claiming generic vertex UBOs, TGSI, OpenGL, Vulkan, or Venus support?

## Applied contract

WebBoxVM recognizes the planar form below, plus one fixed generic-varying form
that declares `IN[1]`/`OUT[1]` as `GENERIC[0]` and appends
`MOV OUT[1], IN[1]` after the same position addition:

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
positions and triangles again before generating the existing solid or texture
material packet. The generic form retains fixed position/UV input; when paired
with texture×fragment-constant, it translates before expanding the local copy
to the existing 40-byte texture-color snapshot. Consequently, post-queue
resource mutation cannot change deferred CPU/WebGPU work.

## Native proof

The AArch64 guest creates one 36-byte R8 constant buffer. It sends isolated
command-9 writes for fragment RGBA at byte four and `[-.015625,0,0,0]` at byte
20, reads all 36 bytes back, then binds command-27 stage zero at byte 20 and
stage one at byte four. The host harness checks the transformed schema-2 vertex
bits, completes the queue, and requires source-over BGRA triangle samples of
`147,141,58,255`. A direct Rust standard-stream regression separately checks
the generic form with texture×fragment-constant, including its translated
schema-8 position and constant-color fields. The later guest `VGM1` phase
reuses that stage-0 range for its generic position/UV texture-constant record;
its native execution remains subject to the same boot-time evidence limit.

## Deliberate limits

There is no matrix, array, arbitrary instruction, generic vertex constant,
second UBO slot, non-16-byte range, clip-space repair, or fallback transform.
The one optional generic varying is fixed to the existing position/UV material
layouts; this remains a vertical seam, not a broad shader implementation.

## Sources

- [VirGL protocol command layout](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/virgl_protocol.h)
- [virglrenderer uniform-buffer decoder](https://android.googlesource.com/platform/external/virglrenderer/+/5fbaeb95136674231649045fa197897ad9bd3ff4/src/vrend_decode.c)
