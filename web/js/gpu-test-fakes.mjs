export function fakeStatus() {
  return { dataset: {}, textContent: "" };
}

export function fakeCanvas({ canvas2d = false, webgpu = false } = {}) {
  const context2d = {
    clears: [],
    images: [],
    clearRect(...args) { this.clears.push(args); },
    createImageData(width, height) {
      return { data: new Uint8ClampedArray(width * height * 4), height, width };
    },
    putImageData(image, x, y) { this.images.push({ image, x, y }); },
  };
  const contextGpu = {
    configureCalls: [],
    configure(options) { this.configureCalls.push(options); },
    getCurrentTexture() { return { createView: () => ({ kind: "canvas-view" }) }; },
  };
  return {
    context2d,
    contextGpu,
    dataset: {},
    height: 480,
    width: 640,
    getContext(kind) {
      if (kind === "2d" && canvas2d) return context2d;
      if (kind === "webgpu" && webgpu) return contextGpu;
      return null;
    },
  };
}

export function fakeGpu(adapters) {
  return {
    requestCount: 0,
    getPreferredCanvasFormat: () => "bgra8unorm",
    async requestAdapter() {
      const adapter = adapters[this.requestCount];
      this.requestCount += 1;
      return adapter;
    },
  };
}

export function fakeAdapter(device, isFallbackAdapter = false, info) {
  return { info, isFallbackAdapter, requestDevice: async () => device };
}

export function fakeDevice({ scopeErrors = [], workDone = Promise.resolve() } = {}) {
  let lose;
  const lost = new Promise((resolve) => { lose = resolve; });
  const scopeStack = [];
  const device = {
    bindGroups: [],
    bufferWrites: [],
    buffers: [],
    draw: [],
    drawIndexed: [],
    limits: { maxTextureDimension2D: 8192 },
    lost,
    pipelines: [],
    pipelineAsyncCalls: 0,
    renderPasses: [],
    scopePops: [],
    scopePushes: [],
    submits: 0,
    textures: [],
    writes: [],
    createBindGroup(descriptor) {
      this.bindGroups.push(descriptor);
      return descriptor;
    },
    createBuffer(descriptor) {
      const buffer = resource(descriptor);
      this.buffers.push(buffer);
      return buffer;
    },
    createCommandEncoder() {
      return {
        beginRenderPass(descriptor) {
          device.renderPasses.push(descriptor);
          const pass = {
            descriptor,
            draw(count) { device.draw.push(count); },
            drawIndexed(count) { device.drawIndexed.push(count); },
            end() {},
            setBindGroup() {},
            setIndexBuffer() {},
            setPipeline() {},
            setVertexBuffer() {},
          };
          return pass;
        },
        finish: () => ({ kind: "commands" }),
      };
    },
    createRenderPipeline(descriptor) {
      const pipeline = { descriptor, getBindGroupLayout: () => ({ kind: "layout" }) };
      this.pipelines.push(pipeline);
      return pipeline;
    },
    async createRenderPipelineAsync(descriptor) {
      this.pipelineAsyncCalls += 1;
      return this.createRenderPipeline(descriptor);
    },
    createSampler: () => ({ kind: "sampler" }),
    createShaderModule: () => ({ kind: "shader" }),
    createTexture(descriptor) {
      const texture = resource(descriptor);
      texture.createView = () => ({ kind: "texture-view" });
      this.textures.push(texture);
      return texture;
    },
    lose,
    popErrorScope() {
      this.scopePops.push(scopeStack.pop());
      return Promise.resolve(scopeErrors.shift() ?? null);
    },
    pushErrorScope(filter) {
      scopeStack.push(filter);
      this.scopePushes.push(filter);
    },
  };
  device.queue = {
    onSubmittedWorkDone: () => workDone,
    submit() { device.submits += 1; },
    writeBuffer(buffer, offset, data) {
      device.bufferWrites.push({ buffer, data: copyBytes(data), offset });
    },
    writeTexture(destination, data, layout, size) {
      device.writes.push({ data: new Uint8Array(data), destination, layout, size });
    },
  };
  return device;
}

function resource(descriptor) {
  return {
    descriptor,
    destroy() { this.destroyed = true; },
  };
}

function copyBytes(data) {
  return new Uint8Array(data.buffer, data.byteOffset, data.byteLength).slice();
}
