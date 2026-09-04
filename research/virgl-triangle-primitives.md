# Research: bounded VirGL triangle primitives

## Question

How can WebBoxVM accept common standard VirGL triangle topologies without
adding a browser-facing topology or weakening packet validation?

## Findings

VirGL renderer capsets advertise triangle lists, triangle strips, and triangle
fans through the Gallium primitive mask. Mesa expands a strip into successive
triples while alternating its first two indices: `[0,1,2]`, `[2,1,3]`,
`[2,3,4]`. Its normal non-flat fan convention retains the first spoke:
`[0,1,2]`, `[0,2,3]`, `[0,3,4]`.

## Application

WebBoxVM advertises and accepts only list bit 4, strip bit 5, and fan bit 6.
A `DRAW_VBO` source count is bounded to 3 through 1,023. Lists must be
divisible by three; strips and fans may use every count in that range. Rust
resolves VBO or index values, expands them to a list, then validates and
snapshots the resulting vertices exactly as it already does for triangle lists.

The invariant is: every accepted strip window or fan spoke creates exactly one
ordered, nondegenerate output triple. The result has `3 * (N - 2)` vertices,
at most 3,063; that is also the browser envelope limit. CPU rasterization and
WebGPU receive identical list packets, with no topology-specific pipeline or
unbounded source access.

## Deliberate limits

Lines, points, primitive restart, instancing, and arbitrary topologies remain
unsupported and unadvertised. The native Linux smoke test still demonstrates
triangle lists; strips and fans have bounded Rust and browser-packet coverage.
This is not general Mesa/OpenGL support.

## Sources

- [VirGL renderer capset primitive mask](https://android.googlesource.com/platform/external/virglrenderer/+/68429e8e1106d0861d9f9f180583bd8381b8bf96/src/vrend_renderer.c)
- [Mesa softpipe primitive expansion](https://android.googlesource.com/platform/external/chromium_org/third_party/mesa/src/+/6da96107e2467063c72e1ec5804f0618a6ce83d3/src/gallium/drivers/softpipe/sp_prim_vbuf.c)
