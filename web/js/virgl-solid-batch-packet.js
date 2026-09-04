const MAGIC = [0x56, 0x47, 0x42, 0x31];
const HEADER_BYTES = 48;
const DRAW_STATE_BYTES = 60;
const MAX_DIMENSION = 8192;
const MAX_DRAWS = 16;
const MAX_VERTICES = 3063;

export function isVirglSolidBatchPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglSolidBatchSequence(packet) {
  if (!isVirglSolidBatchPacket(packet) || packet.byteLength < 12) return undefined;
  return new DataView(packet.buffer, packet.byteOffset, packet.byteLength).getUint32(8, true) || undefined;
}

export function parseVirglSolidBatchPacket(packet) {
  if (!isVirglSolidBatchPacket(packet) || packet.byteLength < HEADER_BYTES) {
    throw new Error("VirGL solid-batch packet has invalid VGB1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, drawCount, flags] = [4, 8, 12, 16, 20, 24]
    .map((offset) => view.getUint32(offset, true));
  if (![1, 2].includes(version) || !sequence || drawCount < 2 || drawCount > MAX_DRAWS || flags !== 0) {
    throw new Error("VirGL solid-batch packet has invalid VGB1 framing");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL solid-batch dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 28, "clear");
  const depthClear = view.getFloat32(44, true);
  if (depthClear !== (version === 2 ? 1 : 0)) throw new Error("VirGL solid-batch depth clear is invalid");
  const draws = [];
  let offset = HEADER_BYTES;
  let totalVertices = 0;
  for (let index = 0; index < drawCount; index += 1) {
    if (offset + DRAW_STATE_BYTES > packet.byteLength) throw new Error("VirGL solid-batch draw is truncated");
    const vertexCount = view.getUint32(offset, true);
    if (vertexCount < 3 || vertexCount > MAX_VERTICES || vertexCount % 3 !== 0) {
      throw new Error("VirGL solid-batch vertex count is invalid");
    }
    totalVertices += vertexCount;
    if (totalVertices > MAX_DRAWS * MAX_VERTICES) throw new Error("VirGL solid-batch vertex budget is invalid");
    const drawColor = colors(view, offset + 4, "draw");
    const viewport = readFloats(view, offset + 20, 6);
    validateViewport(viewport, canvasWidth, canvasHeight);
    const scissor = readScissor(view, offset + 44, canvasWidth, canvasHeight);
    const vertexBytes = vertexCount * 16;
    const next = offset + DRAW_STATE_BYTES + vertexBytes;
    if (next > packet.byteLength) throw new Error("VirGL solid-batch vertices are truncated");
    const vertices = readFloats(view, offset + DRAW_STATE_BYTES, vertexCount * 4);
    if (!validPositions(vertices)) throw new Error("VirGL solid-batch vertices are invalid");
    draws.push({ drawColor, scissor, vertexCount, vertices, viewport });
    offset = next;
  }
  if (offset !== packet.byteLength) throw new Error("VirGL solid-batch packet has trailing bytes");
  const depth = version === 2;
  return {
    acceleration: depth ? "webgpu-virgl-capset1-depth-batch" : "webgpu-virgl-capset1-solid-batch",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, depthClear, draws,
    presentationLabel: depth ? "VirGL capset 1 depth-tested draw batch" : "VirGL capset 1 solid draw batch",
    protocol: depth ? "virgl-depth-batch" : "virgl-solid-batch", sequence, version,
  };
}

function colors(view, offset, label) {
  const values = readFloats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL solid-batch ${label} color must be normalized and finite`);
  }
  return values;
}

function validPositions(vertices) {
  if (!vertices.every((value, index) => {
    const component = index % 4;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1 : value === 1;
  })) return false;
  for (let base = 0; base < vertices.length; base += 12) {
    const [ax, ay] = vertices.subarray(base, base + 2);
    const [bx, by] = vertices.subarray(base + 4, base + 6);
    const [cx, cy] = vertices.subarray(base + 8, base + 10);
    if (Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) < 0.001) return false;
  }
  return true;
}

function validateViewport(viewport, width, height) {
  const [sx, sy, sz, tx, ty, tz] = viewport;
  if (!viewport.every(Number.isFinite) || sx <= 0 || sy <= 0 || sz < 0
    || tx - sx < 0 || tx + sx > width || ty - sy < 0 || ty + sy > height
    || tz - sz < 0 || tz + sz > 1) throw new Error("VirGL solid-batch viewport must fit its target");
}

function readScissor(view, offset, width, height) {
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12]
    .map((delta) => view.getUint32(offset + delta, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return undefined;
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL solid-batch scissor must fit its target");
  }
  return { height: scissorHeight, width: scissorWidth, x, y };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
