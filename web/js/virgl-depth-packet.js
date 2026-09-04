const MAGIC = [0x56, 0x47, 0x44, 0x31];
const MAX_DIMENSION = 8192;
const MAX_VERTEX_COUNT = 3063;
const VERTICES_PER_TRIANGLE = 3;

export function parseVirglDepthPacket(packet) {
  if (!(packet instanceof Uint8Array) || packet.byteLength < 24
    || !MAGIC.every((byte, index) => packet[index] === byte)) {
    throw new Error("VirGL depth packet has invalid VGD1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, vertexCount] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  const state = 56 + vertexCount * 16;
  if (version !== 9 || !sequence || !validCount(vertexCount) || packet.byteLength !== state + 44) {
    throw new Error("VirGL depth packet has invalid VGD1 framing");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL depth dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const drawColor = colors(view, 40, "draw");
  const vertices = readFloats(view, 56, vertexCount * 4);
  if (!validPositions(vertices)) throw new Error("VirGL depth vertices are invalid");
  const depthClear = view.getFloat32(state + 40, true);
  if (depthClear !== 1) throw new Error("VirGL depth clear must be exactly one");
  return {
    acceleration: "webgpu-virgl-capset1-depth", canvasHeight, canvasWidth, capsetId: 1,
    clearColor, depthClear, drawColor, presentationLabel: "VirGL capset 1 depth-tested triangles",
    protocol: "virgl-depth", sequence, version, vertexCount, vertices,
    ...viewportState(view, canvasWidth, canvasHeight, state),
  };
}

function validCount(count) {
  return count >= VERTICES_PER_TRIANGLE && count <= MAX_VERTEX_COUNT && count % VERTICES_PER_TRIANGLE === 0;
}

function colors(view, offset, label) {
  const values = readFloats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL depth ${label} color must contain normalized finite values`);
  }
  return values;
}

function validPositions(vertices) {
  const valid = vertices.every((value, index) => {
    const component = index % 4;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1 : value === 1;
  });
  for (let base = 0; valid && base < vertices.length; base += 12) {
    const [ax, ay] = vertices.subarray(base, base + 2);
    const [bx, by] = vertices.subarray(base + 4, base + 6);
    const [cx, cy] = vertices.subarray(base + 8, base + 10);
    if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) return false;
  }
  return valid;
}

function viewportState(view, width, height, state) {
  const viewport = readFloats(view, state, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL depth viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12].map((offset) => view.getUint32(state + 24 + offset, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL depth scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
