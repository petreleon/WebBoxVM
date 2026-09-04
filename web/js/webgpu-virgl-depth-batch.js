import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-readback-pool-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";
import { submitTextureReadback } from "./webgpu-readback.js?v=20260904-virgl-readback-pool-r1";

const SHADER = `
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f) -> Output {
  var output: Output; output.position = position; output.position.z = (position.z + position.w) * 0.5; output.color = color; return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return input.color; }
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglDepthBatchRenderer {
  #bufferUsage; #depthTexture; #generation = 0; #height = 0; #pipelines = new Map(); #revision = 0;
  #session; #textureUsage; #vertexBuffer; #vertexCapacity = 0; #width = 0;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#textureUsage = options.textureUsage ?? globalThis.GPUTextureUsage ?? { RENDER_ATTACHMENT: 0x10 };
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") throw new Error("WebGPU queue completion tracking is unavailable");
    if (!isCurrent()) return false;
    try {
      const states = frame.draws.map((draw) => ({
        depthCompare: draw.depthCompare ?? frame.depthCompare,
        depthWriteEnabled: draw.depthWriteEnabled ?? frame.depthWriteEnabled,
      }));
      if (!await this.#ensurePipelines(backend, states) || !isCurrent()) return false;
      const revision = this.#revision;
      const readback = await captureWebGpuErrors(device, () => this.#issueDraw(backend, frame));
      return revision === this.#revision && isCurrent() && (readback ? { readback } : true);
    } catch (error) { this.invalidate(); throw error; }
  }

  invalidate() {
    this.#revision += 1;
    this.#vertexBuffer?.destroy?.(); this.#depthTexture?.destroy?.();
    this.#vertexBuffer = undefined; this.#depthTexture = undefined; this.#pipelines.clear();
    this.#generation = 0; this.#vertexCapacity = 0; this.#width = 0; this.#height = 0;
  }

  async #ensurePipelines(backend, states) {
    if (this.#generation !== backend.deviceGeneration) {
      this.invalidate(); this.#generation = backend.deviceGeneration;
    }
    const missing = [...new Map(states.map((state) => [pipelineKey(state), state])).values()]
      .filter((state) => !this.#pipelines.has(pipelineKey(state)));
    if (!missing.length) return true;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL capset 1 depth-batch shader" });
    for (const { depthCompare, depthWriteEnabled } of missing) {
      const pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
        depthStencil: { depthCompare, depthWriteEnabled, format: "depth24plus" },
        fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
        label: "VirGL capset 1 depth-batch pipeline", layout: "auto", primitive: { topology: "triangle-list" },
        vertex: { buffers: [{ arrayStride: 32, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }] }], entryPoint: "vertex_main", module },
      }));
      if (revision !== this.#revision) return false;
      this.#pipelines.set(pipelineKey({ depthCompare, depthWriteEnabled }), pipeline);
    }
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend;
    const vertices = interleave(frame.draws);
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      vertices.byteLength, "VirGL capset 1 depth-batch vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight); this.#ensureDepth(backend, frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#vertexBuffer, 0, vertices);
    const encoder = device.createCommandEncoder({ label: "VirGL capset 1 depth-batch encoder" });
    const target = backend.canvasContext.getCurrentTexture();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] }, loadOp: "clear", storeOp: "store", view: target.createView() }],
      depthStencilAttachment: { depthClearValue: frame.depthClear, depthLoadOp: "clear", depthStoreOp: "store", view: this.#depthTexture.createView() },
      label: "VirGL capset 1 depth-batch pass",
    });
    pass.setVertexBuffer(0, this.#vertexBuffer);
    let first = 0;
    for (const draw of frame.draws) {
      const pipeline = this.#pipelines.get(pipelineKey({
        depthCompare: draw.depthCompare ?? frame.depthCompare,
        depthWriteEnabled: draw.depthWriteEnabled ?? frame.depthWriteEnabled,
      }));
      if (!pipeline) throw new Error("VirGL depth-batch pipeline is unavailable");
      pass.setPipeline(pipeline);
      pass.setViewport(...webGpuViewport(draw, frame.canvasHeight));
      const scissor = draw.scissor ?? { x: 0, y: 0, width: frame.canvasWidth, height: frame.canvasHeight };
      pass.setScissorRect(scissor.x, scissor.y, scissor.width, scissor.height);
      pass.draw(draw.vertexCount, 1, first); first += draw.vertexCount;
    }
    pass.end();
    return submitTextureReadback(device, encoder, target, frame.canvasWidth, frame.canvasHeight, backend.format);
  }

  #ensureDepth(backend, width, height) {
    if (this.#depthTexture && this.#width === width && this.#height === height) return;
    this.#depthTexture?.destroy?.();
    this.#depthTexture = backend.device.createTexture({ format: "depth24plus", label: "VirGL capset 1 depth batch", size: { depthOrArrayLayers: 1, height, width }, usage: this.#textureUsage.RENDER_ATTACHMENT ?? 0x10 });
    this.#width = width; this.#height = height;
  }
}

function interleave(draws) {
  const vertices = new Float32Array(draws.reduce((total, draw) => total + draw.vertexCount, 0) * 8);
  let offset = 0;
  for (const draw of draws) for (let source = 0; source < draw.vertices.length; source += 4) {
    vertices.set(draw.vertices.subarray(source, source + 4), offset); vertices.set(draw.drawColor, offset + 4); offset += 8;
  }
  return vertices;
}

function pipelineKey({ depthCompare, depthWriteEnabled }) {
  return `${depthCompare}:${depthWriteEnabled}`;
}

function webGpuViewport(draw, height) {
  const [sx, sy, sz, tx, ty, tz] = draw.viewport;
  return [tx - sx, height - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
