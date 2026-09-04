const MAGIC = [0x56, 0x47, 0x42, 0x31];
const HEADER_BYTES = 48;
const REPLACEMENT_HEADER_BYTES = 52;
const DRAW_STATE_BYTES = 60;
const DRAW_COMPARE_BYTES = 4;
const MAX_DIMENSION = 8192;
const MAX_DRAWS = 16;
const MAX_VERTICES = 3063;
const RGB_WRITE_MASK = 7;
const DEPTH_COMPARE = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];

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
  const residentCandidate = [6, 7].includes(version); const replace = [8, 9, 10, 11].includes(version);
  const writeMask = [10, 11].includes(version) ? RGB_WRITE_MASK : 0xF;
  if (![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].includes(version) || !sequence || drawCount < 1 || drawCount > MAX_DRAWS
    || (!residentCandidate && !replace && drawCount < 2)
    || (residentCandidate ? flags !== 1 : version !== 3 && flags !== 0)) {
    throw new Error("VirGL solid-batch packet has invalid VGB1 framing");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL solid-batch dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const residentPreviousProducer = version === 7 ? view.getUint32(48, true) : undefined;
  if (version === 7 && (!residentPreviousProducer || residentPreviousProducer === sequence)) {
    throw new Error("VirGL solid-batch replacement producer is invalid");
  }
  const clearColor = colors(view, 28, "clear");
  const depthClear = view.getFloat32(44, true);
  const depth = ![1, 6, 7, 8, 10].includes(version);
  if (depthClear !== (depth ? 1 : 0)) throw new Error("VirGL solid-batch depth clear is invalid");
  const depthCompare = version === 2 ? "less" : version === 3 ? DEPTH_COMPARE[flags] : undefined;
  if (depth && version < 4 && !depthCompare) throw new Error("VirGL solid-batch depth comparison is invalid");
  const draws = [];
  const stateBytes = DRAW_STATE_BYTES + ([4, 5, 9, 11].includes(version) ? DRAW_COMPARE_BYTES : 0);
  let offset = version === 7 ? REPLACEMENT_HEADER_BYTES : HEADER_BYTES;
  let totalVertices = 0;
  for (let index = 0; index < drawCount; index += 1) {
    if (offset + stateBytes > packet.byteLength) throw new Error("VirGL solid-batch draw is truncated");
    const vertexCount = view.getUint32(offset, true);
    if (vertexCount < 3 || vertexCount > MAX_VERTICES || vertexCount % 3 !== 0) {
      throw new Error("VirGL solid-batch vertex count is invalid");
    }
    totalVertices += vertexCount;
    if (totalVertices > MAX_DRAWS * MAX_VERTICES) throw new Error("VirGL solid-batch vertex budget is invalid");
    const depthState = [5, 9, 11].includes(version) ? view.getUint32(offset + 4, true) : undefined;
    if ([5, 9, 11].includes(version) && ((depthState & 1) === 0 || depthState > 31)) {
      throw new Error("VirGL solid-batch depth state is invalid");
    }
    const drawCompare = version === 4 ? DEPTH_COMPARE[view.getUint32(offset + 4, true)]
      : [5, 9, 11].includes(version) ? DEPTH_COMPARE[depthState >> 2] : depthCompare;
    if (depth && !drawCompare) throw new Error("VirGL solid-batch depth comparison is invalid");
    const state = offset + ([4, 5, 9, 11].includes(version) ? DRAW_COMPARE_BYTES : 0);
    const drawColor = colors(view, state + 4, "draw");
    const viewport = readFloats(view, state + 20, 6);
    validateViewport(viewport, canvasWidth, canvasHeight);
    const scissor = readScissor(view, state + 44, canvasWidth, canvasHeight);
    const vertexBytes = vertexCount * 16;
    const next = offset + stateBytes + vertexBytes;
    if (next > packet.byteLength) throw new Error("VirGL solid-batch vertices are truncated");
    const vertices = readFloats(view, offset + stateBytes, vertexCount * 4);
    if (!validPositions(vertices)) throw new Error("VirGL solid-batch vertices are invalid");
    draws.push({ depthCompare: drawCompare, depthWriteEnabled: ![5, 9, 11].includes(version) || (depthState & 2) !== 0, drawColor, scissor, vertexCount, vertices, viewport });
    offset = next;
  }
  if (offset !== packet.byteLength) throw new Error("VirGL solid-batch packet has trailing bytes");
  return {
    acceleration: depth ? "webgpu-virgl-capset1-depth-batch" : "webgpu-virgl-capset1-solid-batch",
    blend: replace ? "replace" : "source-over",
    canvasHeight, canvasWidth, capsetId: 1, clearColor, depthClear, depthCompare, depthWriteEnabled: true, draws, writeMask,
    presentationLabel: depth ? "VirGL capset 1 depth-tested draw batch" : "VirGL capset 1 solid draw batch",
    protocol: depth ? "virgl-depth-batch" : "virgl-solid-batch", residentCandidate, residentPreviousProducer, sequence, version,
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
