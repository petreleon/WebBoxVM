import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-readback-pool-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";

const SHADER = `
struct Scene { matrix: mat4x4<f32>, color: vec4f }
@group(0) @binding(0) var<uniform> scene: Scene;
struct Output { @builtin(position) position: vec4f }
@vertex fn vertex_main(@location(0) position: vec4f) -> Output {
  var output: Output;
  let clip = vec4f(dot(scene.matrix[0], position), dot(scene.matrix[1], position), dot(scene.matrix[2], position), dot(scene.matrix[3], position));
  output.position = clip;
  output.position.z = (clip.z + clip.w) * 0.5;
  return output;
}
@fragment fn fragment_main() -> @location(0) vec4f { return scene.color; }
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglMatrixRenderer {
  #bindGroup; #bufferUsage; #generation = 0; #pipeline; #revision = 0; #scene = new Float32Array(20);
  #session; #uniformBuffer; #vertexBuffer; #vertexCapacity = 0;

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
    } catch (error) {
      this.invalidate();
      throw error;
    }
  }

  invalidate() {
    this.#revision += 1; this.#uniformBuffer?.destroy?.(); this.#vertexBuffer?.destroy?.();
    this.#bindGroup = undefined; this.#uniformBuffer = undefined; this.#vertexBuffer = undefined;
    this.#pipeline = undefined; this.#generation = 0; this.#vertexCapacity = 0;
  }

  async #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    const revision = this.#revision; const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL capset 1 matrix shader" });
    this.#pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
      label: "VirGL capset 1 matrix pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 16, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }] }], entryPoint: "vertex_main", module },
    }));
    if (revision !== this.#revision) return false;
    await captureWebGpuErrors(device, () => {
      this.#uniformBuffer = device.createBuffer({ label: "VirGL capset 1 DP4 scene", size: 80, usage: this.#bufferUsage.COPY_DST | this.#bufferUsage.UNIFORM });
      this.#bindGroup = device.createBindGroup({ entries: [{ binding: 0, resource: { buffer: this.#uniformBuffer } }], layout: this.#pipeline.getBindGroupLayout(0) });
    });
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    const { device } = backend;
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      frame.vertices.byteLength, "VirGL capset 1 matrix positions", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#session.configure(frame.canvasWidth, frame.canvasHeight); this.#scene.set(frame.matrix); this.#scene.set(frame.drawColor, 16);
    device.queue.writeBuffer(this.#uniformBuffer, 0, this.#scene); device.queue.writeBuffer(this.#vertexBuffer, 0, frame.vertices);
    const encoder = device.createCommandEncoder({ label: "VirGL capset 1 matrix encoder" });
    const pass = encoder.beginRenderPass({
      colorAttachments: [{ clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] }, loadOp: "clear", storeOp: "store", view: backend.canvasContext.getCurrentTexture().createView() }],
      label: "VirGL capset 1 matrix pass",
    });
    pass.setPipeline(this.#pipeline); pass.setBindGroup(0, this.#bindGroup); pass.setVertexBuffer(0, this.#vertexBuffer);
    const viewport = webGpuViewport(frame); if (viewport) pass.setViewport(...viewport);
    if (frame.scissor) pass.setScissorRect(frame.scissor.x, frame.scissor.y, frame.scissor.width, frame.scissor.height);
    pass.draw(frame.vertexCount); pass.end(); device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }
}

function webGpuViewport(frame) {
  if (!frame.viewport) return undefined;
  const [sx, sy, sz, tx, ty, tz] = frame.viewport;
  return [tx - sx, frame.canvasHeight - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
