# Exact VirGL vertex-upload reuse

## Question

How can repeated bounded VirGL frames avoid a redundant WebGPU vertex upload
without letting a changed guest packet render stale geometry?

## Contract

The solid and material batch renderers retain one CPU byte snapshot for their
current GPU vertex buffer. A call skips `queue.writeBuffer` only when both are
true:

1. the exact current `GPUBuffer` is still live; and
2. every byte of the newly packed vertex data equals the retained snapshot.

Any byte difference, buffer growth/replacement, renderer invalidation, or
WebGPU device-generation change forces a normal upload. The retained snapshot
is capped at 2 MiB; oversized batches remain correct but upload every time.

The equality check is deliberately byte exact. A hash-only cache could collide
and silently show old guest geometry, which is not an acceptable rendering
contract.

## Cost boundary

The cache does not remove packet parsing or draw submission. It avoids the
host-to-GPU copy for static repeated geometry, including resident output
redraws. It is complementary to texture-snapshot caching and resident target
reuse: neither assumes that the other owns a vertex buffer.

## Validation

Tests prove that an identical payload skips one upload, a one-byte mutation
uploads again, a new GPU buffer uploads even with identical bytes, and renderer
tests cover both solid and mixed-material batches.

## Scope

This is a private browser optimization for the bounded VirGL packet subset. It
does not add guest-visible buffer residency, generic VirGL/OpenGL support,
Vulkan, or Venus semantics.

Source: [WebGPU `GPUQueue.writeBuffer`](https://www.w3.org/TR/webgpu/#dom-gpuqueue-writebuffer).
