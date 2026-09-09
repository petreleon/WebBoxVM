# F06.1 graphics boundary map

This map assigns ownership; it does not claim stock Mesa, Venus, or a public API profile.

## Public directions

| Root | Owns | May depend on | Must not own |
| --- | --- | --- | --- |
| `emulator/src/devices/virtio_gpu/` | VirtIO wire validation, resources, completion | typed emulator memory/IRQ APIs | browser or WebGPU objects |
| `three_d/{pending,residency,transfer}` | protocol-neutral queued-work/coherence state | validated VirtIO resources | shader text or browser lifecycle |
| `three_d/virgl/{stream,shader,draw}` | bounded VirGL parse, shader classification, packets | shared 3D state | WebGPU calls or Mesa compatibility claims |
| `web/js/gpu-3d-packet.js` | browser packet framing/negative validation | protocol envelope data | guest-memory mutation |
| `web/js/webgpu-3d*.js` | browser platform routing, WebGPU lifecycle, device loss | validated browser frames | VirtIO decoding or guest-driver policy |
| `guest/*` | freestanding transport fixtures | guest UAPI headers | host-private behavior as a standard claim |
| `emulator/src/devices/virtio_gpu/tests` and `web/js/*.test.mjs` | boundary and regression fixtures | public boundaries only | hidden production state |

The browser facade `ExperimentalWebGpu3dRenderer` imports only the VirGL route dispatcher and
the private WBG3 lifecycle owner. `webgpu-3d-dispatch.js` owns bounded VirGL protocol routing;
`webgpu-3d-legacy.js` owns the WBG3 shader/pipeline/buffer lifecycle. Neither changes the guest
wire parser or implies that WBG3 is a standard capability.

## Current seam ownership

| Existing seam | Owner | Explicit limit |
| --- | --- | --- |
| Capsets 1/2 and private 7 | Rust protocol/3D boundary | no Venus capset 4 |
| VirGL stream, TGSI subset, packets | Rust shader frontend/backend | bounded shapes only |
| Blob/shadow/WBL1 preparation | Rust shared runtime | not Venus protocol/Vulkan memory |
| GPU packet parsing and completion | browser protocol/platform | no guest-memory completion without acknowledgment |
| WebGPU output targets/readback | browser platform | async, device-generation-safe bounded route |
| WBG3 fallback draw | browser WBG3 lifecycle | private protocol, no Mesa evidence |
| Guest clear demos and matrix tests | guest/test fixtures | not stock Mesa applications |

## Transition rules

1. New browser WebGPU work enters a dedicated owner module; the facade only constructs, routes,
   invalidates, and releases it.
2. A shader frontend produces validated protocol-neutral work; a backend may not reach guest memory.
3. Generated protocol files must carry F02.3 provenance and cannot become public handwritten APIs.
4. A cross-root test names the public behavior it observes. Tests may use fixtures but cannot certify
   an unimplemented API profile or hardware route.
5. A missing root needs an explicit child task before use; F02/F06 children establish source and
   generated-output roots before Venus code generation begins.
