import { VirglRouteDispatcher } from "./webgpu-3d-dispatch.js?v=20260904-virgl-readback-pool-r1";
import { LegacyWbg3Renderer } from "./webgpu-3d-legacy.js?v=20260904-virgl-readback-pool-r1";

/**
 * Public browser-platform facade for guest 3D packets.
 *
 * Protocol-specific VirGL routing and the private WBG3 lifecycle deliberately
 * live behind separate owners. Callers retain this stable facade only.
 */
export class ExperimentalWebGpu3dRenderer {
  #legacy;
  #virgl;

  constructor(session, options = {}) {
    this.#legacy = new LegacyWbg3Renderer(session, options);
    this.#virgl = new VirglRouteDispatcher(session, options);
  }

  render(backend, frame, isCurrent = () => true) {
    return this.#virgl.render(backend, frame, isCurrent)
      ?? this.#legacy.render(backend, frame, isCurrent);
  }

  invalidate() {
    this.#virgl.invalidate();
    this.#legacy.invalidate();
  }

  release(frame) {
    this.#virgl.release(frame);
  }
}
