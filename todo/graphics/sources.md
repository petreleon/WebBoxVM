# Primary sources and feasibility gates

Access date: 2026-09-09. Recheck moving specifications and pin upstream revisions
before implementing a protocol or publishing a compatibility profile.

## Sources

1. [Mesa VirGL](https://docs.mesa3d.org/drivers/virgl.html)
   describes the real Gallium-based virtual GPU and its TGSI shader representation.
   Treat stock Mesa interoperability as a separate gate from private GPU commands.
2. [Mesa Venus](https://docs.mesa3d.org/drivers/venus.html)
   defines Venus as Vulkan command serialization and lists guest requirements for
   3D features, capset query fix, blob resources, host-visible resources and context
   initialization; its host-visible memory discussion identifies mapping and
   coherence assumptions that cannot simply be assumed in a browser.
3. [Upstream Venus protocol](https://gitlab.freedesktop.org/virgl/venus-protocol)
   is the protocol and code-generation repository linked by Mesa; Mesa also links
   the upstream virglrenderer implementation as the renderer reference.
   Repository identity was verified through Mesa, but direct source inspection
   was blocked by the site's browser challenge, so no source revision was verified.
4. [WebGPU specification source](https://raw.githubusercontent.com/gpuweb/gpuweb/main/spec/index.bs)
   restricts MAP_READ buffers to COPY_DST and MAP_WRITE buffers to COPY_SRC,
   defines asynchronous mapping and queue completion, and validates device features
   and limits explicitly; guest memory, synchronization and feature advertisement
   therefore require an intentional translation model.
5. [WGSL specification](https://www.w3.org/TR/WGSL/)
   defines vertex, fragment and compute stages, with its own type, memory-layout,
   control-flow uniformity and synchronization rules.
   TGSI/SPIR-V translation and any geometry/tessellation lowering need semantic
   tests, rather than a shader-text substitution or an assumed stage equivalence.
6. [Khronos VK-GL-CTS](https://github.com/KhronosGroup/VK-GL-CTS)
   contains tests for Vulkan, OpenGL, OpenGL ES and EGL, and emits inspectable QPA
   logs; record pinned test versions, exact case lists and every skip or failure.
   A selected test subset is useful development evidence, not full conformance.
7. [Khronos Vulkan Registry](https://registry.khronos.org/vulkan/)
   provides the API registry, specification and validation sources, and distinguishes
   freely available CTS code from the Khronos Adopter requirements for use of the
   Vulkan trademark; local correctness testing and formal approval are separate gates.
8. [Vulkan portability subset](https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_portability_subset.html)
   exposes specified deviations for implementations layered over other APIs.
   It is an explicitly limited profile, not permission to omit arbitrary mandatory
   behavior or claim full conformance without the required functionality.

## Engineering conclusions to verify

Draft final API targets, subject to explicit feasibility analysis and profile freeze:
[OpenGL 4.6 core](https://registry.khronos.org/OpenGL/specs/gl/glspec46.core.pdf),
[OpenGL ES 3.2](https://registry.khronos.org/OpenGL/specs/es/3.2/es_spec_3.2.pdf),
and [Vulkan 1.4](https://docs.vulkan.org/guide/latest/versions.html).
These are planning defaults, not statements that browser implementation or
near-native execution of all required behavior has been demonstrated.
Earlier-version bring-up is an intermediate gate; a claimed final API version
requires its mandatory functionality and limits, plus every advertised extension.
OpenGL compatibility-profile behavior and all optional extensions are not implied
by the core-profile target; track additional promises individually in the matrix.

- Pin stock Mesa, Linux virtio-gpu, Venus protocol and renderer reference versions;
  capture actual guest wire traffic and compare it with the reference implementation.
  Passing a custom guest demo cannot close the stock-driver interoperability gate.
- Prototype host-visible memory, flush/invalidate, queue ordering, fences, semaphores,
  readback and presentation early; guest-visible completion must track actual GPU work.
- Build an API/format/shader-feature matrix against the actual browser adapter.
  Classify each entry as native mapping, proven emulation, unsupported or unresolved,
  and advertise only the supported semantics of the chosen profile.
- Optional compute lowering or software emulation may cover selected gaps; this is
  an engineering option, not a performance finding. Measure extra passes, copies,
  CPU time and synchronization costs, and report fallback separately.
- Near-native guest performance remains a measured objective: compare the same
  complete application workload on the same hardware against a declared native
  baseline, including emulated guest CPU, driver, translation, GPU and presentation.
  Native WebGPU replay can isolate overhead but cannot replace the guest benchmark.
