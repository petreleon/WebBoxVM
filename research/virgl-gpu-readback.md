# Bounded VirGL GPU readback

## Decision

`VGM1` material and `VGB1` solid batches can make their final color target GPU-authoritative.
The browser renders the bounded batch, copies that same canvas texture into a
mapped buffer, and delivers its tightly packed pixels with the submission
sequence. Rust accepts the payload only for a matching in-flight `VGM1`/`VGB1`
completion and writes it into the matching guest color resource.

This replaces CPU color replay for that one envelope. It is not a general
VirGL, OpenGL, Vulkan, Venus, or zero-copy resource implementation.

## Browser path

```text
VGM1/VGB1 -> WebGPU render pass -> canvas texture (RENDER_ATTACHMENT | COPY_SRC)
     -> copyTextureToBuffer -> MAP_READ | COPY_DST buffer -> mapAsync(READ)
     -> tightly packed pixels + sequence -> VM worker -> Rust completion
```

The canvas is configured with both `RENDER_ATTACHMENT` and `COPY_SRC`.
`copyTextureToBuffer` uses a 256-byte-aligned `bytesPerRow`; after mapping, the
browser strips row padding before transferring a bounded byte view. Format tag
1 means `bgra8unorm`; tag 2 means `rgba8unorm`, which Rust converts to its
canonical BGRA resource representation.

The WebGPU specification requires `COPY_SRC` on the source texture, `COPY_DST`
on the destination buffer, and permits `MAP_READ` only with `COPY_DST`.
It also requires aligned texture-buffer copy rows. `mapAsync()` is the fence
used for the payload, rather than treating queue submission alone as readable.

## Completion and failure contract

- The Rust pending entry must have been emitted as `VGM1` or `VGB1`, delivered to
  the browser, and have a live completion record on the matching fence timeline.
- The payload format, exact `rect.width * rect.height * 4` byte count, resource
  kind, target bounds, and VirGL context generation are checked before writes.
- Bad/stale/misordered payloads complete the saved guest request with an error;
  they do not apply the CPU effect or mutate the color target.
- Browsers without a mappable copy path retain the prior Boolean acknowledgment,
  which deliberately uses the existing bounded CPU replay instead.

Depth `VGM1`/`VGB1` batches preserve a bounded CPU depth update, then replace
the color target with the mapped GPU pixels. Thus final guest-visible color is
GPU-authoritative while depth remains a compatibility seam, not GPU depth
readback. The operation rolls back color/depth snapshots if the depth update or
color replacement cannot complete.

## Bound and performance implications

Readback is capped at 64 MiB, matching the resource ceiling. It transfers
`4 * width * height` tightly packed bytes plus up to one padded row per source
row on the GPU copy. It removes CPU rasterization from this path but adds a
GPU-to-CPU synchronization point and a worker transfer; it therefore needs an
end-to-end benchmark before any near-native performance claim.

The next useful optimization is an explicit guest-visible resource residency
contract: retain GPU-resident targets across draws and read back only when the
guest requests CPU visibility. That requires broader synchronization/state
semantics and is intentionally outside this bounded change.

## Validation boundary

Repository tests cover padded-row removal, delayed map completion, both channel
formats, browser-worker forwarding, strict Rust target validation, error-on-
malformed payload, guest completion, and CPU depth preservation with GPU color
replacement. They do not prove a hardware browser run or a performance gain.

## Sources

- [W3C WebGPU: buffer usages](https://www.w3.org/TR/webgpu/#buffer-usage)
- [W3C WebGPU: texture usages](https://www.w3.org/TR/webgpu/#texture-usages)
- [W3C WebGPU: texture-to-buffer copies](https://www.w3.org/TR/webgpu/#dom-gpucommandencoder-copytexturetobuffer)
- [W3C WebGPU: mapped buffer ranges](https://www.w3.org/TR/webgpu/#dom-gpubuffer-mapasync)
