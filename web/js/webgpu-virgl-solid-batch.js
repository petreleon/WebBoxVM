import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-readback-pool-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";
import { virglColorTarget } from "./webgpu-virgl-color-target.js?v=20260904-virgl-readback-pool-r1";
import { submitTextureReadback } from "./webgpu-readback.js?v=20260904-virgl-readback-pool-r1";
import { VirglResidentOutputTargets } from "./webgpu-virgl-output-target.js?v=20260904-virgl-readback-pool-r1";
import { VirglVertexUploadCache } from "./webgpu-virgl-vertex-cache.js?v=20260904-virgl-readback-pool-r1";

const SHADER = `
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f) -> Output {
  var output: Output;
  output.position = position;
  output.position.z = (position.z + position.w) * 0.5;
  output.color = color;
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return input.color; }
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglSolidBatchRenderer {
  #bufferUsage; #generation = 0; #outputs; #pipelines = new Map(); #revision = 0; #session; #vertexBuffer; #vertexCapacity = 0; #vertices = new VirglVertexUploadCache();

  constructor(session, options = {}, outputs) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#outputs = outputs ?? new VirglResidentOutputTargets();
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") throw new Error("WebGPU queue completion tracking is unavailable");
    if (!isCurrent()) return false;
    try {
      if (!await this.#ensurePipeline(backend, frame) || !isCurrent()) return false;
      const revision = this.#revision; const output = this.#outputs.acquire(backend, frame);
      const readback = await captureWebGpuErrors(device, () => this.#issueDraw(backend, frame, output));
      if (revision !== this.#revision || !isCurrent()) { this.#outputs.abandon(output); return false; }
      return output ? this.#outputs.publish(backend, output) && { resident: true } : readback ? { readback } : true;
    } catch (error) { this.invalidate(); throw error; }
  }

  async readback(backend, frame, isCurrent) {
    const output = this.#outputs.get(backend, frame);
    if (!output || !isCurrent()) return false;
    try {
      const readback = await captureWebGpuErrors(backend.device, () => submitTextureReadback(
        backend.device, backend.device.createCommandEncoder({ label: "VirGL resident readback" }), output.texture,
        frame.canvasWidth, frame.canvasHeight, backend.format,
      ));
      if (!readback || !isCurrent()) return false;
      this.#outputs.release(frame.producerSequence);
      return { readback };
    } catch (error) { this.#outputs.release(frame.producerSequence); this.invalidate(); throw error; }
  }

  release(frame) { this.#outputs.release(frame.producerSequence); }

  invalidate() {
    this.#outputs.invalidate();
    this.#invalidatePipeline();
  }

  #invalidatePipeline() {
    this.#revision += 1;
    this.#vertexBuffer?.destroy?.();
    this.#vertices.invalidate();
    this.#pipelines.clear(); this.#vertexBuffer = undefined; this.#generation = 0; this.#vertexCapacity = 0;
  }

  async #ensurePipeline(backend, frame) {
    if (this.#generation !== backend.deviceGeneration) { this.#invalidatePipeline(); this.#generation = backend.deviceGeneration; }
    const blend = frame.blend ?? "source-over";
    const writeMask = frame.writeMask ?? 0xF; const key = `${blend}:${writeMask}`;
    if (this.#pipelines.has(key)) return true;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL capset 1 solid-batch shader" });
    const pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      fragment: { entryPoint: "fragment_main", module, targets: [virglColorTarget(backend.format, blend, writeMask, SOURCE_OVER)] },
      label: "VirGL capset 1 solid-batch pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 32, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }] }], entryPoint: "vertex_main", module },
    }));
    if (revision !== this.#revision) return false;
    this.#pipelines.set(key, pipeline); return true;
  }

  #issueDraw(backend, frame, output) {
    const { device } = backend;
    const vertices = interleave(frame.draws);
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      vertices.byteLength, "VirGL capset 1 solid-batch vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    this.#vertices.upload(device, this.#vertexBuffer, vertices);
    const encoder = device.createCommandEncoder({ label: "VirGL capset 1 solid-batch encoder" });
    const target = output?.texture ?? backend.canvasContext.getCurrentTexture();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] }, loadOp: "clear", storeOp: "store", view: target.createView() }],
      label: "VirGL capset 1 solid-batch pass",
    });
    const pipeline = this.#pipelines.get(`${frame.blend ?? "source-over"}:${frame.writeMask ?? 0xF}`);
    if (!pipeline) throw new Error("VirGL solid-batch pipeline is unavailable");
    pass.setPipeline(pipeline); pass.setVertexBuffer(0, this.#vertexBuffer);
    let first = 0;
    for (const draw of frame.draws) {
      pass.setViewport(...webGpuViewport(draw, frame.canvasHeight));
      const scissor = draw.scissor ?? { x: 0, y: 0, width: frame.canvasWidth, height: frame.canvasHeight };
      pass.setScissorRect(scissor.x, scissor.y, scissor.width, scissor.height);
      pass.draw(draw.vertexCount, 1, first); first += draw.vertexCount;
    }
    pass.end();
    if (output) {
      if (typeof encoder.copyTextureToTexture !== "function") throw new Error("WebGPU texture copies are unavailable");
      encoder.copyTextureToTexture({ texture: target }, { texture: backend.canvasContext.getCurrentTexture() }, {
        depthOrArrayLayers: 1, height: frame.canvasHeight, width: frame.canvasWidth,
      });
      device.queue.submit([encoder.finish()]);
      return Promise.resolve(device.queue.onSubmittedWorkDone());
    }
    return submitTextureReadback(device, encoder, target, frame.canvasWidth, frame.canvasHeight, backend.format);
  }
}

function interleave(draws) {
  const floats = new Float32Array(draws.reduce((total, draw) => total + draw.vertexCount, 0) * 8);
  let offset = 0;
  for (const draw of draws) {
    for (let source = 0; source < draw.vertices.length; source += 4) {
      floats.set(draw.vertices.subarray(source, source + 4), offset);
      floats.set(draw.drawColor, offset + 4); offset += 8;
    }
  }
  return new Uint8Array(floats.buffer);
}

function webGpuViewport(draw, height) {
  const [sx, sy, sz, tx, ty, tz] = draw.viewport;
  return [tx - sx, height - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
