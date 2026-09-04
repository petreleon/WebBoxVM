# VirGL texture snapshot cache

## Purpose

Bounded `VGM1` packets carry immutable sampled-texture snapshots. Recreating a
WebGPU texture, uploading it, and destroying it for every material batch wastes
work when a guest reuses unchanged texture data across draws.

## Invariant

An entry is reusable only when its width, height, byte length, and every BGRA
byte equal the incoming snapshot. A fast FNV-1a hash chooses a small bucket;
the byte comparison prevents a hash collision from changing rendered output.

An evicted texture is not destroyed while a command encoder may reference it.
It moves to the current batch's retired list and is destroyed only after that
batch's readback promise settles. Device invalidation clears cached textures.

## Bounds and complexity

The cache holds at most 32 textures and 4 MiB of copied snapshot bytes. Lookup
is expected O(B + P), where B is a usually single collision bucket and P is the
snapshot byte count for an exact equality check. Insertion can scan O(N) to
choose the oldest of N <= 32 entries, which is bounded and off the guest CPU
hot path.

Sampler objects are cached by the finite accepted `(addressMode, filter)` pair.
This reduces repeated descriptor allocation without extending the guest ABI.
For an exact cached snapshot tuple and the same material pipeline, the matching
bind group is also reused. The cache holds at most 64 bind groups and clears all
of them before any texture eviction can leave a descriptor naming a retired
texture.

## Boundary

This is browser-side caching of already-copied private packet snapshots. It is
not guest GPU-resource residency, zero-copy transfer, general VirGL texture
support, Mesa/OpenGL compatibility, Vulkan, or Venus external memory.

## Validation

The material-batch browser test presents two byte-identical packets and proves
one source texture upload, one vertex upload, and one texture bind group. Cache
tests also prove a changed snapshot creates a new binding and eviction drops old
bindings before reuse. Full browser tests retain packet parsing, submission,
readback, and device-loss coverage.

Source: [W3C WebGPU resource lifetime](https://www.w3.org/TR/webgpu/#resource-lifetime).
