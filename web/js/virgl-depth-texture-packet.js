const MAGIC = [0x56, 0x47, 0x44, 0x31];
const MAX_DIMENSION = 8192;
const MAX_TEXTURE_DIMENSION = 64;
const MAX_VERTEX_COUNT = 3063;
const DEPTH_COMPARE = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];

export function parseVirglDepthTexturePacket(packet) {
  if (!(packet instanceof Uint8Array) || packet.byteLength < 24
    || !MAGIC.every((byte, index) => packet[index] === byte)) {
    throw new Error("VirGL depth-texture packet has invalid VGD1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, vertexCount] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  const state = 56 + vertexCount * 24;
  if (version !== 13 || !sequence || !validCount(vertexCount) || packet.byteLength < state + 60) {
    throw new Error("VirGL depth-texture packet has invalid VGD1 framing");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL depth-texture dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const reserved = colors(view, 40, "reserved");
  if (![...reserved].every((value) => value === 0)) throw new Error("VirGL depth-texture reserved color must be zero");
  const vertices = floats(view, 56, vertexCount * 6);
  if (!validPositions(vertices) || !validUvs(vertices)) throw new Error("VirGL depth-texture vertices are invalid");
  const sampler = samplerConfig(view.getUint32(state + 40, true));
  const width = view.getUint32(state + 44, true);
  const height = view.getUint32(state + 48, true);
  const bytes = textureBytes(width, height);
  const tail = state + 52 + bytes;
  if (!sampler || !bytes || packet.byteLength !== tail + 8) {
    throw new Error("VirGL depth-texture packet has invalid sampler or texture framing");
  }
  const depthClear = view.getFloat32(tail, true);
  const dsa = view.getUint32(tail + 4, true);
  if (depthClear !== 1) throw new Error("VirGL depth-texture depth clear must be exactly one");
  if ((dsa & 1) === 0 || dsa > 31) throw new Error("VirGL depth-texture depth state must be canonical");
  return {
    acceleration: "webgpu-virgl-capset1-depth-texture", canvasHeight, canvasWidth, capsetId: 1,
    clearColor, depthClear, depthCompare: DEPTH_COMPARE[dsa >> 2], depthWriteEnabled: (dsa & 2) !== 0,
    drawColor: reserved, presentationLabel: "VirGL capset 1 depth-tested textured triangles",
    protocol: "virgl-depth-texture", sequence, texture: { ...sampler, height, pixels: packet.subarray(state + 52, tail), width },
    version, vertexCount, vertices, ...viewportState(view, canvasWidth, canvasHeight, state),
  };
}

function validCount(count) {
  return count >= 3 && count <= MAX_VERTEX_COUNT && count % 3 === 0;
}

function colors(view, offset, label) {
  const values = floats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL depth-texture ${label} color must contain normalized finite values`);
  }
  return values;
}

function validPositions(vertices) {
  const valid = vertices.every((value, index) => {
    const component = index % 6;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1 : component === 3 ? value === 1 : true;
  });
  for (let base = 0; valid && base < vertices.length; base += 18) {
    const [ax, ay] = vertices.subarray(base, base + 2);
    const [bx, by] = vertices.subarray(base + 6, base + 8);
    const [cx, cy] = vertices.subarray(base + 12, base + 14);
    if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) return false;
  }
  return valid;
}

function validUvs(vertices) {
  return vertices.every((value, index) => index % 6 < 4 || Number.isFinite(value) && value >= -8 && value <= 8);
}

function viewportState(view, width, height, state) {
  const viewport = floats(view, state, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL depth-texture viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12].map((offset) => view.getUint32(state + 24 + offset, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL depth-texture scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function samplerConfig(word) {
  if (word === 0x1092) return { addressMode: "clamp-to-edge", filter: "nearest" };
  if (word === 0x1080) return { addressMode: "repeat", filter: "nearest" };
  if (word === 0x3292) return { addressMode: "clamp-to-edge", filter: "linear" };
}

function textureBytes(width, height) {
  return width && height && width <= MAX_TEXTURE_DIMENSION && height <= MAX_TEXTURE_DIMENSION ? width * height * 4 : 0;
}

function floats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
