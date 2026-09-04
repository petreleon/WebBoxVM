# Research: bounded VirGL triangle strips

## Question

How can WebBoxVM accept a common standard VirGL primitive without adding a new
browser-facing topology or weakening packet validation?

## Findings

VirGL renderer capsets advertise triangle lists, triangle strips, and triangle
fans through the Gallium primitive mask. Mesa's software primitive assembler
turns a strip into successive triples while alternating the first two indices:
`[0,1,2]`, `[2,1,3]`, `[2,3,4]`, and so on. That preserves each triangle's
winding while sharing the strip's adjacent vertices.

## Application

WebBoxVM advertises and accepts only list bit 4 and strip bit 5. A `DRAW_VBO`
source count is bounded to 3 through 1,023. Lists must be divisible by three;
strips may use every count in that range. Rust first resolves VBO or index
values, expands strips to a list, then validates and snapshots the resulting
vertices exactly as it already does for triangle lists.

The invariant is: every input strip window contributes exactly one ordered,
nondegenerate output triple, and the output has `3 * (N - 2)` vertices. Its
maximum is therefore 3,063, which is also the browser envelope limit. CPU
rasterization and WebGPU receive identical list packets; neither needs a
strip-specific pipeline or unbounded source access.

## Deliberate limits

Triangle fans, lines, points, primitive restart, instancing, and arbitrary
topologies remain unsupported and unadvertised. The native Linux smoke test
still demonstrates triangle lists; the strip route is covered by bounded Rust
and browser-packet tests. This is not general Mesa/OpenGL support.

## Sources

- [VirGL renderer capset primitive mask](https://android.googlesource.com/platform/external/virglrenderer/+/af3d1de900fb0bf9094ed5d6ed361d8c2c71a543/src/vrend_renderer.c)
- [Mesa softpipe strip expansion](https://android.googlesource.com/platform/external/chromium_org/third_party/mesa/src/+/6da96107e2467063c72e1ec5804f0618a6ce83d3/src/gallium/drivers/softpipe/sp_prim_vbuf.c)
