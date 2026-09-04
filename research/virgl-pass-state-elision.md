# VirGL material-pass state elision

## Purpose

Bounded `VGM1` batches may contain up to 16 ordered draws. Repeating WebGPU
state setters for unchanged contiguous draws adds JavaScript-to-GPU command
encoding work without changing the resulting pixels.

## Invariant

Within one render pass, the renderer remembers the last pipeline, bind group,
viewport, and scissor it encoded. It emits a setter only when the next draw's
exact value differs. Vertex-buffer selection and `draw` remain per-draw because
their offsets and counts can differ.

A pipeline change resets the remembered bind group. The renderer therefore
rebinds a texture group after any intervening pipeline, even if the cached
group object is byte-identical. This makes the optimization independent of
bind-group persistence or layout compatibility across WebGPU pipelines.

## Cost model

For a contiguous run of `N` equal material states, state encoding changes from
`N` pipeline, bind-group, viewport, and scissor calls to one each. The worst
case remains bounded by the existing 16-draw packet cap, and draw ordering is
unchanged.

## Validation

Browser tests prove that two matching texture-color draws emit each retained
state once, while a texture-solid-texture sequence emits two texture bindings
around its pipeline transition. The full packet, readback, and device-loss
suite continues to cover rendered completion behavior.

## Boundary

This only reduces browser command encoding in the existing private, bounded
VirGL subset. It does not add a guest ABI, general OpenGL/VirGL support,
Vulkan, or Venus semantics.

Source: [WebGPU render-pass encoding](https://www.w3.org/TR/webgpu/#render-pass-encoding).
