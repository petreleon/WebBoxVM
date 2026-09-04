import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-gpu-readback-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-gpu-readback-r1";

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
  #bufferUsage; #generation = 0; #pipeline; #revision = 0; #session; #vertexBuffer; #vertexCapacity = 0;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
  }

  async render(backend, frame, isCurrent) {
    const { device } = backend;
    if (typeof device.queue.onSubmittedWorkDone !== "function") throw new Error("WebGPU queue completion tracking is unavailable");
    if (!isCurrent()) return false;
    try {
      if (!await this.#ensurePipeline(backend) || !isCurrent()) return false;
      const revision = this.#revision;
      await captureWebGpuErrors(device, () => this.#issueDraw(backend, frame));
      return revision === this.#revision && isCurrent();
    } catch (error) { this.invalidate(); throw error; }
  }

  invalidate() {
    this.#revision += 1;
    this.#vertexBuffer?.destroy?.();
    this.#pipeline = undefined; this.#vertexBuffer = undefined; this.#generation = 0; this.#vertexCapacity = 0;
  }

  async #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    const revision = this.#revision;
    const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL capset 1 solid-batch shader" });
    this.#pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
      label: "VirGL capset 1 solid-batch pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 32, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }] }], entryPoint: "vertex_main", module },
    }));
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend;
    const vertices = interleave(frame.draws);
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      vertices.byteLength, "VirGL capset 1 solid-batch vertices", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#vertexBuffer, 0, vertices);
    const encoder = device.createCommandEncoder({ label: "VirGL capset 1 solid-batch encoder" });
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] }, loadOp: "clear", storeOp: "store", view: backend.canvasContext.getCurrentTexture().createView() }],
      label: "VirGL capset 1 solid-batch pass",
    });
    pass.setPipeline(this.#pipeline); pass.setVertexBuffer(0, this.#vertexBuffer);
    let first = 0;
    for (const draw of frame.draws) {
      pass.setViewport(...webGpuViewport(draw, frame.canvasHeight));
      const scissor = draw.scissor ?? { x: 0, y: 0, width: frame.canvasWidth, height: frame.canvasHeight };
      pass.setScissorRect(scissor.x, scissor.y, scissor.width, scissor.height);
      pass.draw(draw.vertexCount, 1, first); first += draw.vertexCount;
    }
    pass.end(); device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }
}

function interleave(draws) {
  const vertices = new Float32Array(draws.reduce((total, draw) => total + draw.vertexCount, 0) * 8);
  let offset = 0;
  for (const draw of draws) {
    for (let source = 0; source < draw.vertices.length; source += 4) {
      vertices.set(draw.vertices.subarray(source, source + 4), offset);
      vertices.set(draw.drawColor, offset + 4); offset += 8;
    }
  }
  return vertices;
}

function webGpuViewport(draw, height) {
  const [sx, sy, sz, tx, ty, tz] = draw.viewport;
  return [tx - sx, height - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
