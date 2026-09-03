const MAGIC = [0x56, 0x47, 0x44, 0x31];
const FIXED_BYTES = 228;
const MAX_DIMENSION = 8192;
const MAX_TEXTURE_DIMENSION = 64;
const VERTEX_COUNT = 3;

export function parseVirglTextureColorPacket(packet) {
  if (!(packet instanceof Uint8Array)) throw new TypeError("VirGL texture-color packet must be a Uint8Array");
  if (packet.byteLength < FIXED_BYTES || !MAGIC.every((byte, index) => packet[index] === byte)) {
    throw new Error("VirGL texture-color packet has invalid VGD1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, vertexCount] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  if (version !== 8 || sequence === 0 || vertexCount !== VERTEX_COUNT) {
    throw new Error("VirGL texture-color packet has an invalid version, sequence, or vertex count");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL texture-color dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const reserved = colors(view, 40, "reserved");
  if (![...reserved].every((value) => value === 0)) throw new Error("VirGL texture-color reserved color must be zero");
  const vertices = readFloats(view, 56, VERTEX_COUNT * 10);
  if (!validVertices(vertices)) throw new Error("VirGL texture-color vertices must be bounded and normalized");
  const state = viewportState(view, canvasWidth, canvasHeight);
  const sampler = samplerConfig(view.getUint32(216, true));
  const [width, height] = [220, 224].map((offset) => view.getUint32(offset, true));
  const bytes = width * height * 4;
  if (!sampler || !width || !height || width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION
    || packet.byteLength !== FIXED_BYTES + bytes) {
    throw new Error("VirGL texture-color packet has invalid sampler or texture framing");
  }
  return {
    acceleration: "webgpu-virgl-capset1-texture-color", canvasHeight, canvasWidth, capsetId: 1,
    clearColor, drawColor: reserved, presentationLabel: "VirGL capset 1 texture-color triangle",
    protocol: "virgl-texture-color", sequence, version, vertexCount, vertices, ...state,
    texture: { width, height, pixels: packet.subarray(FIXED_BYTES), ...sampler },
  };
}

function colors(view, offset, label) {
  const values = readFloats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL texture-color ${label} color must contain normalized finite values`);
  }
  return values;
}

function validVertices(vertices) {
  const valid = vertices.every((value, index) => {
    const component = index % 10;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1
      : component === 3 ? value === 1
        : component < 8 ? Number.isFinite(value) && value >= 0 && value <= 1
          : Number.isFinite(value) && value >= -8 && value <= 8;
  });
  const [ax, ay] = vertices;
  const [bx, by] = vertices.subarray(10);
  const [cx, cy] = vertices.subarray(20);
  return valid && Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) >= 0.001;
}

function viewportState(view, width, height) {
  const viewport = readFloats(view, 176, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height
    && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL texture-color viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12]
    .map((offset) => view.getUint32(200 + offset, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL texture-color scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function samplerConfig(word) {
  if (word === 0x1092) return { addressMode: "clamp-to-edge", filter: "nearest" };
  if (word === 0x1080) return { addressMode: "repeat", filter: "nearest" };
  if (word === 0x3292) return { addressMode: "clamp-to-edge", filter: "linear" };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
