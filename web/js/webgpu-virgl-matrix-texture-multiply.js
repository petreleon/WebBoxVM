import { defaultBufferUsage, ensureBuffer } from "./webgpu-3d-resources.js?v=20260904-virgl-readback-pool-r1";
import { captureWebGpuErrors } from "./webgpu-errors.js?v=20260904-virgl-readback-pool-r1";
import { padBgraRows } from "./gpu-scanout-packet.js?v=20260904-virgl-readback-pool-r1";
import { copyTextureIfChanged } from "./webgpu-virgl-texture-upload.js?v=20260904-virgl-readback-pool-r1";

const SHADER = `
struct Scene { matrix: mat4x4<f32> }
@group(0) @binding(0) var<uniform> scene: Scene;
@group(0) @binding(1) var left: texture_2d<f32>;
@group(0) @binding(2) var right: texture_2d<f32>;
@group(0) @binding(3) var leftSampler: sampler;
@group(0) @binding(4) var rightSampler: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) uv: vec2f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) uv: vec2f) -> Output {
  var output: Output;
  let clip = vec4f(dot(scene.matrix[0], position), dot(scene.matrix[1], position), dot(scene.matrix[2], position), dot(scene.matrix[3], position));
  output.position = clip;
  output.position.z = (clip.z + clip.w) * 0.5;
  output.uv = uv;
  return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f {
  let uv = vec2f(input.uv.x, 1.0 - input.uv.y);
  return textureSampleLevel(left, leftSampler, uv, 0.0) * textureSampleLevel(right, rightSampler, uv, 0.0);
}
`;
const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export class VirglMatrixTextureMultiplyRenderer {
  #bindGroup; #bufferUsage; #generation = 0; #pipeline; #revision = 0; #samplerKeys = []; #samplers = []; #session;
  #texturePixels = []; #textureSizes = []; #textures = []; #textureUsage; #uniformBuffer; #vertexBuffer; #vertexCapacity = 0;

  constructor(session, options = {}) {
    this.#session = session;
    this.#bufferUsage = options.bufferUsage ?? globalThis.GPUBufferUsage ?? defaultBufferUsage();
    this.#textureUsage = options.textureUsage ?? globalThis.GPUTextureUsage ?? { COPY_DST: 0x2, TEXTURE_BINDING: 0x4 };
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
      this.invalidate(); throw error;
    }
  }

  invalidate() {
    this.#revision += 1; this.#uniformBuffer?.destroy?.(); this.#vertexBuffer?.destroy?.(); this.#textures.forEach((texture) => texture?.destroy?.());
    this.#bindGroup = this.#uniformBuffer = this.#vertexBuffer = undefined; this.#pipeline = undefined;
    this.#samplerKeys = []; this.#samplers = []; this.#texturePixels = []; this.#textureSizes = []; this.#textures = [];
    this.#generation = this.#vertexCapacity = 0;
  }

  async #ensurePipeline(backend) {
    if (this.#generation === backend.deviceGeneration && this.#pipeline) return true;
    this.invalidate(); this.#generation = backend.deviceGeneration;
    const revision = this.#revision; const { device } = backend;
    if (typeof device.createRenderPipelineAsync !== "function") throw new Error("WebGPU asynchronous pipeline validation is unavailable");
    const module = device.createShaderModule({ code: SHADER, label: "VirGL matrix dual-texture shader" });
    const pipeline = await captureWebGpuErrors(device, () => device.createRenderPipelineAsync({
      fragment: { entryPoint: "fragment_main", module, targets: [{ blend: SOURCE_OVER, format: backend.format }] },
      label: "VirGL matrix dual-texture pipeline", layout: "auto", primitive: { topology: "triangle-list" },
      vertex: { buffers: [{ arrayStride: 24, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x2", offset: 16, shaderLocation: 1 }] }], entryPoint: "vertex_main", module },
    }));
    if (revision !== this.#revision) return false;
    this.#pipeline = pipeline;
    await captureWebGpuErrors(device, () => {
      this.#uniformBuffer = device.createBuffer({ label: "VirGL matrix dual-texture rows", size: 64, usage: this.#bufferUsage.COPY_DST | this.#bufferUsage.UNIFORM });
    });
    return revision === this.#revision;
  }

  #issueDraw(backend, frame) {
    if (frame.textures?.length !== 2) throw new Error("VirGL matrix dual-texture frame needs two textures");
    const { device } = backend;
    [this.#vertexBuffer, this.#vertexCapacity] = ensureBuffer(device, this.#vertexBuffer, this.#vertexCapacity,
      frame.vertices.byteLength, "VirGL matrix dual-texture attributes", this.#bufferUsage.COPY_DST | this.#bufferUsage.VERTEX);
    this.#ensureSamplers(device, frame.textures); this.#ensureTextures(device, frame.textures); this.#session.configure(frame.canvasWidth, frame.canvasHeight);
    device.queue.writeBuffer(this.#uniformBuffer, 0, frame.matrix); device.queue.writeBuffer(this.#vertexBuffer, 0, frame.vertices);
    frame.textures.forEach((texture, index) => this.#upload(device, this.#textures[index], texture, index));
    const encoder = device.createCommandEncoder({ label: "VirGL matrix dual-texture encoder" });
    const pass = encoder.beginRenderPass({ colorAttachments: [{
      clearValue: { r: frame.clearColor[0], g: frame.clearColor[1], b: frame.clearColor[2], a: frame.clearColor[3] },
      loadOp: "clear", storeOp: "store", view: backend.canvasContext.getCurrentTexture().createView(),
    }], label: "VirGL matrix dual-texture pass" });
    pass.setPipeline(this.#pipeline); pass.setBindGroup(0, this.#bindGroup); pass.setVertexBuffer(0, this.#vertexBuffer);
    const viewport = webGpuViewport(frame); if (viewport) pass.setViewport(...viewport);
    if (frame.scissor) pass.setScissorRect(frame.scissor.x, frame.scissor.y, frame.scissor.width, frame.scissor.height);
    pass.draw(frame.vertexCount); pass.end(); device.queue.submit([encoder.finish()]);
    return device.queue.onSubmittedWorkDone();
  }

  #upload(device, target, texture, index) {
    const pixels = copyTextureIfChanged(this.#texturePixels[index], texture.pixels); if (!pixels) return;
    const upload = padBgraRows(pixels, texture.width, texture.height);
    device.queue.writeTexture({ texture: target }, upload.data, { bytesPerRow: upload.bytesPerRow, rowsPerImage: texture.height }, { width: texture.width, height: texture.height, depthOrArrayLayers: 1 });
    this.#texturePixels[index] = pixels;
  }

  #ensureTextures(device, textures) {
    const same = this.#textures.length === 2 && textures.every((texture, index) => this.#textures[index] && this.#textureSizes[index]?.[0] === texture.width && this.#textureSizes[index][1] === texture.height);
    if (!same) {
      this.#textures.forEach((texture) => texture.destroy?.()); this.#textureSizes = textures.map(({ width, height }) => [width, height]); this.#texturePixels = []; this.#bindGroup = undefined;
      this.#textures = [];
      for (const { width, height } of textures) {
        this.#textures.push(device.createTexture({ format: "bgra8unorm", label: "VirGL matrix sampled texture", size: { width, height, depthOrArrayLayers: 1 }, usage: this.#textureUsage.COPY_DST | this.#textureUsage.TEXTURE_BINDING }));
      }
    }
    if (!this.#bindGroup) this.#bindGroup = device.createBindGroup({ entries: [
      { binding: 0, resource: { buffer: this.#uniformBuffer } }, { binding: 1, resource: this.#textures[0].createView() }, { binding: 2, resource: this.#textures[1].createView() }, { binding: 3, resource: this.#samplers[0] }, { binding: 4, resource: this.#samplers[1] },
    ], layout: this.#pipeline.getBindGroupLayout(0) });
  }

  #ensureSamplers(device, textures) {
    const keys = textures.map(({ addressMode = "clamp-to-edge", filter = "nearest" }) => [addressMode, filter]);
    if (keys.every(([addressMode, filter], index) => this.#samplerKeys[index]?.[0] === addressMode && this.#samplerKeys[index]?.[1] === filter)) return;
    this.#samplers = keys.map(([addressMode, filter]) => device.createSampler({ addressModeU: addressMode, addressModeV: addressMode, magFilter: filter, minFilter: filter, mipmapFilter: "nearest" }));
    this.#samplerKeys = keys; this.#bindGroup = undefined;
  }
}

function webGpuViewport(frame) {
  if (!frame.viewport) return undefined;
  const [sx, sy, sz, tx, ty, tz] = frame.viewport;
  return [tx - sx, frame.canvasHeight - ty - sy, sx * 2, sy * 2, tz - sz, tz + sz];
}
