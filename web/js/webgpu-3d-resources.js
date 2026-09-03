export function defaultBufferUsage() {
  return { COPY_DST: 0x08, INDEX: 0x10, UNIFORM: 0x40, VERTEX: 0x20 };
}

export function ensureBuffer(device, buffer, capacity, bytes, label, usage) {
  const needed = nextPowerOfTwo(Math.max(4, bytes));
  if (buffer && capacity >= needed) return [buffer, capacity];
  buffer?.destroy?.();
  return [device.createBuffer({ label, size: needed, usage }), needed];
}

export function paddedIndexBytes(indices) {
  const source = new Uint8Array(indices.buffer, indices.byteOffset, indices.byteLength);
  const result = new Uint8Array((source.byteLength + 3) & ~3);
  result.set(source);
  return result;
}

export function pipelineDescriptor(module, format) {
  return {
    depthStencil: { depthCompare: "less", depthWriteEnabled: true, format: "depth24plus" },
    fragment: { entryPoint: "fragment_main", module, targets: [{ format }] },
    label: "experimental guest 3D pipeline",
    layout: "auto",
    primitive: { topology: "triangle-list" },
    vertex: {
      buffers: [{
        arrayStride: 7 * 4,
        attributes: [
          { format: "float32x3", offset: 0, shaderLocation: 0 },
          { format: "float32x4", offset: 12, shaderLocation: 1 },
        ],
      }],
      entryPoint: "vertex_main",
      module,
    },
  };
}

export function renderPassDescriptor(backend, depthTexture, frame) {
  return {
    colorAttachments: [{
      clearValue: {
        a: frame.clearColor[3], b: frame.clearColor[2],
        g: frame.clearColor[1], r: frame.clearColor[0],
      },
      loadOp: "clear", storeOp: "store",
      view: backend.canvasContext.getCurrentTexture().createView(),
    }],
    depthStencilAttachment: {
      depthClearValue: 1, depthLoadOp: "clear", depthStoreOp: "store",
      view: depthTexture.createView(),
    },
    label: "experimental guest 3D pass",
  };
}

function nextPowerOfTwo(value) {
  return 2 ** Math.ceil(Math.log2(value));
}
