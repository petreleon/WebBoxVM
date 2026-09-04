import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-readback-pool-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";
import { submitTextureReadback } from "./webgpu-readback.js?v=20260904-virgl-readback-pool-r1";
import { SOURCE_OVER, materialShader, materialTextures, materialVertexLayout } from "./webgpu-virgl-material-batch-shaders.js?v=20260904-virgl-readback-pool-r1";
import { VirglTextureSnapshotCache } from "./webgpu-virgl-texture-cache.js?v=20260904-virgl-readback-pool-r1";

export class VirglMaterialBatchRenderer {
  #bufferUsage; #depthTexture; #generation = 0; #height = 0; #pipelines = new Map(); #revision = 0;
  #session; #textureUsage; #textures; #vertexBuffer; #vertexCapacity = 0; #width = 0;

  constructor(session, options = {}) {
    this.#session = session; this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    const usage = options.textureUsage ?? globalThis.GPUTextureUsage ?? {};
    this.#textureUsage = { COPY_DST: usage.COPY_DST ?? 2, RENDER_ATTACHMENT: usage.RENDER_ATTACHMENT ?? 16, TEXTURE_BINDING: usage.TEXTURE_BINDING ?? 4 };
    this.#textures = new VirglTextureSnapshotCache(options);
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") throw new Error("WebGPU queue completion tracking is unavailable");
    if (!isCurrent()) return false;
    try {
      if (!await this.#ensurePipelines(backend, frame) || !isCurrent()) return false;
      const revision = this.#revision; const readback = await captureWebGpuErrors(device, () => this.#issueDraw(backend, frame));
      return revision === this.#revision && isCurrent() && (readback ? { readback } : true);
    } catch (error) { this.invalidate(); throw error; }
  }

  invalidate() {
    this.#revision += 1; this.#vertexBuffer?.destroy?.(); this.#depthTexture?.destroy?.();
    this.#textures.invalidate();
    this.#depthTexture = undefined; this.#generation = 0; this.#height = 0; this.#pipelines.clear();
    this.#vertexBuffer = undefined; this.#vertexCapacity = 0; this.#width = 0;
  }

  async #ensurePipelines(backend, frame) {
    if (this.#generation !== backend.deviceGeneration) { this.invalidate(); this.#generation = backend.deviceGeneration; }
    const missing = [...new Map(frame.draws.map((draw) => [key(frame, draw), draw])).values()]
      .filter((draw) => !this.#pipelines.has(key(frame, draw)));
    if (!missing.length) return true;
    const revision = this.#revision; const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    for (const draw of missing) {
      const module = device.createShaderModule({ code: materialShader(draw.material), label: `VirGL ${draw.material} batch shader` });
      const descriptor = {
        fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
        label: `VirGL ${draw.material} batch pipeline`, layout: "auto", primitive: { topology: "triangle-list" },
        vertex: { buffers: [materialVertexLayout(draw.material)], entryPoint: "vertex_main", module },
      };
      if (frame.depth) descriptor.depthStencil = { depthCompare: draw.depthCompare, depthWriteEnabled: draw.depthWriteEnabled, format: "depth24plus" };
      const pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync(descriptor));
      if (revision !== this.#revision) return false;
      this.#pipelines.set(key(frame, draw), pipeline);
    }
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend; const packed = pack(frame.draws);
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      packed.bytes.byteLength, "VirGL capset 1 material-batch vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight); device.queue.writeBuffer(this.#vertexBuffer, 0, packed.bytes);
    const retired = []; const draws = frame.draws.map((draw, index) => ({ ...draw, ...packed.draws[index] }));
    for (const draw of draws) draw.bindGroup = this.#bindGroup(device, this.#pipelines.get(key(frame, draw)), draw, retired);
    const encoder = device.createCommandEncoder({ label: "VirGL capset 1 material-batch encoder" }); const target = backend.canvasContext.getCurrentTexture();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] }, loadOp: "clear", storeOp: "store", view: target.createView() }],
      depthStencilAttachment: frame.depth ? this.#depthAttachment(backend, frame) : undefined,
      label: "VirGL capset 1 material-batch pass",
    });
    for (const draw of draws) {
      const pipeline = this.#pipelines.get(key(frame, draw)); if (!pipeline) throw new Error("VirGL material-batch pipeline is unavailable");
      pass.setPipeline(pipeline); if (draw.bindGroup) pass.setBindGroup(0, draw.bindGroup);
      pass.setVertexBuffer(0, this.#vertexBuffer, draw.offset, draw.bytes); pass.setViewport(...viewport(draw, frame.canvasHeight));
      const scissor = draw.scissor ?? { x: 0, y: 0, width: frame.canvasWidth, height: frame.canvasHeight };
      pass.setScissorRect(scissor.x, scissor.y, scissor.width, scissor.height); pass.draw(draw.vertexCount);
    }
    pass.end();
    return submitTextureReadback(device, encoder, target, frame.canvasWidth, frame.canvasHeight, backend.format)
      .finally(() => retired.forEach((texture) => texture.destroy?.()));
  }

  #bindGroup(device, pipeline, draw, retired) {
    const textures = materialTextures(draw); if (!textures.length) return undefined;
    if (!pipeline) throw new Error("VirGL material-batch texture pipeline is unavailable");
    const entries = this.#textures.bindGroupEntries(device, textures, retired);
    return device.createBindGroup({ entries, layout: pipeline.getBindGroupLayout(0) });
  }

  #depthAttachment(backend, frame) {
    if (!this.#depthTexture || this.#width !== frame.canvasWidth || this.#height !== frame.canvasHeight) {
      this.#depthTexture?.destroy?.(); this.#depthTexture = backend.device.createTexture({ format: "depth24plus", label: "VirGL capset 1 material batch depth", size: { width: frame.canvasWidth, height: frame.canvasHeight, depthOrArrayLayers: 1 }, usage: this.#textureUsage.RENDER_ATTACHMENT });
      this.#width = frame.canvasWidth; this.#height = frame.canvasHeight;
    }
    return { depthClearValue: frame.depthClear, depthLoadOp: "clear", depthStoreOp: "store", view: this.#depthTexture.createView() };
  }
}

function key(frame, draw) { return frame.depth ? `${draw.material}:${draw.depthCompare}:${draw.depthWriteEnabled}` : draw.material; }

function pack(draws) {
  const sources = draws.map((draw) => draw.material === "solid" ? solidVertices(draw) : rawBytes(draw.vertices));
  const bytes = new Uint8Array(sources.reduce((total, source) => total + source.byteLength, 0)); let offset = 0;
  const packed = sources.map((source) => { const current = { bytes: source.byteLength, offset }; bytes.set(source, offset); offset += source.byteLength; return current; });
  return { bytes, draws: packed };
}

function solidVertices(draw) {
  const vertices = new Float32Array(draw.vertexCount * 8);
  for (let source = 0, target = 0; source < draw.vertices.length; source += 4, target += 8) {
    vertices.set(draw.vertices.subarray(source, source + 4), target); vertices.set(draw.drawColor, target + 4);
  }
  return rawBytes(vertices);
}

function rawBytes(values) { return new Uint8Array(values.buffer, values.byteOffset, values.byteLength); }

function viewport(draw, height) {
  const [sx, sy, sz, tx, ty, tz] = draw.viewport; return [tx - sx, height - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
