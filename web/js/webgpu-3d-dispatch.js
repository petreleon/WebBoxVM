import { renderVirglClear } from "./webgpu-virgl-clear.js?v=20260904-virgl-readback-pool-r1";
import { VirglDepthBatchRenderer } from "./webgpu-virgl-depth-batch.js?v=20260904-virgl-readback-pool-r1";
import { VirglDepthRenderer } from "./webgpu-virgl-depth.js?v=20260904-virgl-readback-pool-r1";
import { VirglDepthTextureColorRenderer } from "./webgpu-virgl-depth-texture-color.js?v=20260904-virgl-readback-pool-r1";
import { VirglDepthTextureRenderer } from "./webgpu-virgl-depth-texture.js?v=20260904-virgl-readback-pool-r1";
import { VirglDrawRenderer } from "./webgpu-virgl-draw.js?v=20260904-virgl-readback-pool-r1";
import { VirglMaterialBatchRenderer } from "./webgpu-virgl-material-batch.js?v=20260904-virgl-readback-pool-r1";
import { VirglMatrixTextureMultiplyRenderer } from "./webgpu-virgl-matrix-texture-multiply.js?v=20260904-virgl-readback-pool-r1";
import { VirglMatrixTextureRenderer } from "./webgpu-virgl-matrix-texture.js?v=20260904-virgl-readback-pool-r1";
import { VirglMatrixVertexColorRenderer } from "./webgpu-virgl-matrix-vertex-color.js?v=20260904-virgl-readback-pool-r1";
import { VirglMatrixRenderer } from "./webgpu-virgl-matrix.js?v=20260904-virgl-readback-pool-r1";
import { VirglResidentOutputTargets } from "./webgpu-virgl-output-target.js?v=20260904-virgl-readback-pool-r1";
import { renderVirglResidentCopy } from "./webgpu-virgl-resident-copy.js?v=20260904-virgl-readback-pool-r1";
import { VirglSolidBatchRenderer } from "./webgpu-virgl-solid-batch.js?v=20260904-virgl-readback-pool-r1";
import { VirglTextureColorRenderer } from "./webgpu-virgl-texture-color.js?v=20260904-virgl-readback-pool-r1";
import { VirglTextureMultiplyRenderer } from "./webgpu-virgl-texture-multiply.js?v=20260904-virgl-readback-pool-r1";
import { VirglTextureRenderer } from "./webgpu-virgl-texture.js?v=20260904-virgl-readback-pool-r1";
import { VirglVertexColorRenderer } from "./webgpu-virgl-vertex-color.js?v=20260904-virgl-readback-pool-r1";

/** Routes bounded standard VirGL packets without owning WBG3 fallback state. */
export class VirglRouteDispatcher {
  #depth; #depthBatch; #depthTexture; #depthTextureColor; #draw; #materialBatch;
  #matrix; #matrixTexture; #matrixTextureMultiply; #matrixVertexColor; #outputs;
  #solidBatch; #texture; #textureColor; #textureMultiply; #vertexColor; #session;

  constructor(session, options = {}) {
    this.#session = session;
    this.#outputs = new VirglResidentOutputTargets();
    this.#draw = new VirglDrawRenderer(session, options);
    this.#matrix = new VirglMatrixRenderer(session, options);
    this.#matrixVertexColor = new VirglMatrixVertexColorRenderer(session, options);
    this.#matrixTexture = new VirglMatrixTextureRenderer(session, options);
    this.#matrixTextureMultiply = new VirglMatrixTextureMultiplyRenderer(session, options);
    this.#solidBatch = new VirglSolidBatchRenderer(session, options, this.#outputs);
    this.#depthBatch = new VirglDepthBatchRenderer(session, options);
    this.#materialBatch = new VirglMaterialBatchRenderer(session, options, this.#outputs);
    this.#depth = new VirglDepthRenderer(session, options);
    this.#depthTexture = new VirglDepthTextureRenderer(session, options);
    this.#depthTextureColor = new VirglDepthTextureColorRenderer(session, options);
    this.#texture = new VirglTextureRenderer(session, options);
    this.#textureMultiply = new VirglTextureMultiplyRenderer(session, options);
    this.#vertexColor = new VirglVertexColorRenderer(session, options);
    this.#textureColor = new VirglTextureColorRenderer(session, options);
  }

  render(backend, frame, isCurrent) {
    if (frame.protocol === "virgl-clear") return renderVirglClear(this.#session, this.#outputs, backend, frame, isCurrent);
    if (frame.protocol === "virgl-draw") return (frame.matrix ? this.#matrix : this.#draw).render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-solid-batch") return this.#solidBatch.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-resident-copy") return renderVirglResidentCopy(this.#outputs, backend, frame, isCurrent);
    if (frame.protocol === "virgl-resident-readback") return this.#solidBatch.readback(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth-batch") return this.#depthBatch.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-material-batch") return this.#materialBatch.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth") return this.#depth.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth-texture") return this.#depthTexture.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-depth-texture-color") return this.#depthTextureColor.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-matrix-texture") return this.#matrixTexture.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-texture") return this.#texture.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-matrix-texture-multiply") return this.#matrixTextureMultiply.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-texture-multiply") return this.#textureMultiply.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-matrix-vertex-color") return this.#matrixVertexColor.render(backend, frame, isCurrent);
    if (["virgl-vertex-color", "virgl-depth-vertex-color"].includes(frame.protocol)) return this.#vertexColor.render(backend, frame, isCurrent);
    if (frame.protocol === "virgl-texture-color") return this.#textureColor.render(backend, frame, isCurrent);
    return undefined;
  }

  invalidate() {
    this.#draw.invalidate(); this.#matrix.invalidate(); this.#matrixVertexColor.invalidate();
    this.#matrixTexture.invalidate(); this.#matrixTextureMultiply.invalidate(); this.#solidBatch.invalidate();
    this.#depthBatch.invalidate(); this.#materialBatch.invalidate(); this.#depth.invalidate();
    this.#depthTexture.invalidate(); this.#depthTextureColor.invalidate(); this.#texture.invalidate();
    this.#textureMultiply.invalidate(); this.#vertexColor.invalidate(); this.#textureColor.invalidate();
  }

  release(frame) {
    this.#solidBatch.release(frame);
  }
}
