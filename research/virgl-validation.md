# VirGL validation evidence

[Compatibility boundaries](virgl-compatibility.md)

## Matrix dual-texture browser check

On 2026-09-09, the v18 browser probe passed on an Apple `metal-3` WebGPU adapter.
All three right-sampler states (`0x1092`, `0x1080`, `0x3292`) completed through
`GuestDisplay`. Real GPU readback returned BGRA `[50,40,200,255]` inside the
transformed triangle and `[77,51,26,255]` outside, with no uncaptured GPU errors.
The one-pixel fixtures validate shader execution and multiplication; they do not
distinguish spatial filtering or establish real guest/Mesa compatibility.

Reproduce with Playwright CLI and the repository web server:

```sh
python3 scripts/serve_web.py --host 127.0.0.1 --port 8765 --directory web
# In another terminal:
playwright-cli --session virgl-v18 open http://127.0.0.1:8765 --headed
playwright-cli --session virgl-v18 run-code --filename scripts/check_virgl_matrix_texture_multiply.mjs
playwright-cli --session virgl-v18 close
```

## Validation retained in the repository

Rust tests prove capset bits, transactional no-clear, malformed-index, inline/resource-constant render/rejection, canonical `DP4` parse/rejection, inline/resource-backed matrix transactional rejection, raw-matrix schema-15 solid, schema-16 generic-RGBA, schema-17 generic-UV texture, and schema-18 dual-texture capture, generic-varying matrix normalization, solid and textured vertex-matrix packet coordinates, and shifted raster output,
exact source-over and sampler setup, rasterizer unbind rejection, bounded batched-triangle, alternating strip, and spoke-preserving fan expansion, schemas 2–14 `VGD1`, bounded `VGB1`, and mixed-material `VGM1`
payloads, singleton and ordered non-depth/depth batch blending, normalized per-vertex RGBA interpolation and texture modulation, repeat-at-one, clamp-linear midpoint, solid/vertex-color/one-texture/texture-color depth, and independent two-sampler CPU sampling, R8G8B8A8-to-BGRA normalization, nonzero-offset indexes, deferred
acknowledgment, clipped source-over raster results, viewport/scissor bounds, non-depth/depth batch ordering, exact `EQUAL` depth, canonical write masks, strict VGB1/VGM1 GPU-color readback, and `WBGF` damage.
Browser tests prove private-envelope framing, malformed sampler rejection, exact
independent WebGPU clamp/repeat/linear descriptors, fixed RGBA and RGBA/UV attributes, one/two padded BGRA uploads, viewport/scissor calls,
cached pipelines, exact byte-identical vertex uploads, row-major v15/v16/v17/v18 matrix shader row-dot paths with v16 RGBA and v17/v18 UV/sampler attributes, and material bind groups, standard singleton material and `VGB1`/`VGM1` shared/per-record depth-batch pipelines, bounded `draw(N)`, one batch render pass, padded map readback, and queue-gated completion.

`scripts/virgl_guest_transport_smoke.sh` provides a native Linux transport harness for
VirtIO-GPU/DRM/KMS transport for the blob profiles, capset discovery, R8 transfer/copy,
clear/fence, indexed inline-constant, texture, vertex-color, texture-color, solid/vertex-color/one-texture/texture-color depth state, and ordered solid-batch paths.
It also creates a 36-byte R8 constant buffer, populates its color at byte offset
four and `[dx,dy,0,0]` at byte offset 20 through two isolated standard command-9 writes plus readback, sends stage-0 and stage-1 command 27 bindings, validates distinct schema-2 `VGD1` color and translated vertices,
completes that effect, reads both `147,141,58,255` triangles through Linux, then expects one standard clear plus two `DRAW_VBO`s as `VGB1` v1 and the ordered `0,128,64,255` center, followed by VGB1 v2 clear-one `LESS`, VGD1 schema 10 `EQUAL`, VGB1 v3 shared-`EQUAL`, v4 `LESS`/`GREATER`, v5 canonical state words `7`/`17`, schema 12 vertex-color DSA word `5`, schema 13 one-texture DSA word `7`, and schema 14 texture-color DSA word `7`.
The newly wired guest phase must then produce one exact 364-byte `VGM1` depth-material batch: a far solid red record followed by a near stage-0-UBO-translated `TEX`×`CONST[0][0]` record reified as texture-color, both DSA word `7`, and a `64,64,64,255` center after acknowledgment. A bounded 55-second native run on 2026-09-04 built the 52,984-byte demo and reached kernel timestamp 18 seconds, but timed out in the shell phase before guest graphics, so native execution of this newest phase remains inconclusive.
This does not claim native Mesa, a native OpenGL context, or browser WebGPU execution
from that harness.
