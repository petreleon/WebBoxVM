const MAGIC = [0x56, 0x47, 0x44, 0x31];
const BYTES = 192;
const MAX_DIMENSION = 8192;
const VERTEX_COUNT = 3;

export function parseVirglVertexColorPacket(packet) {
  if (!(packet instanceof Uint8Array)) throw new TypeError("VirGL vertex-color packet must be a Uint8Array");
  if (packet.byteLength !== BYTES || !MAGIC.every((byte, index) => packet[index] === byte)) {
    throw new Error("VirGL vertex-color packet has invalid VGD1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, canvasWidth, canvasHeight, vertexCount] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  if (version !== 7 || sequence === 0 || vertexCount !== VERTEX_COUNT) {
    throw new Error("VirGL vertex-color packet has an invalid version, sequence, or vertex count");
  }
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL vertex-color dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = colors(view, 24, "clear");
  const reserved = colors(view, 40, "reserved");
  if (![...reserved].every((value) => value === 0)) throw new Error("VirGL vertex-color reserved color must be zero");
  const vertices = readFloats(view, 56, VERTEX_COUNT * 8);
  if (!validPositions(vertices) || !validColors(vertices)) {
    throw new Error("VirGL vertex-color vertices must contain bounded positions and normalized colors");
  }
  const state = viewportState(view, canvasWidth, canvasHeight);
  return {
    acceleration: "webgpu-virgl-capset1-vertex-color", canvasHeight, canvasWidth, capsetId: 1,
    clearColor, drawColor: reserved, presentationLabel: "VirGL capset 1 vertex-color triangle",
    protocol: "virgl-vertex-color", sequence, version, vertexCount, vertices, ...state,
  };
}

function colors(view, offset, label) {
  const values = readFloats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) {
    throw new Error(`VirGL vertex-color ${label} color must contain normalized finite values`);
  }
  return values;
}

function validPositions(vertices) {
  const valid = vertices.every((value, index) => {
    const component = index % 8;
    return component < 3 ? Number.isFinite(value) && value >= -1 && value <= 1
      : component === 3 ? value === 1 : true;
  });
  const [ax, ay] = vertices;
  const [bx, by] = vertices.subarray(8);
  const [cx, cy] = vertices.subarray(16);
  return valid && Math.abs((cx - ax) * (by - ay) - (cy - ay) * (bx - ax)) >= 0.001;
}

function validColors(vertices) {
  return vertices.every((value, index) => index % 8 < 4
    || Number.isFinite(value) && value >= 0 && value <= 1);
}

function viewportState(view, width, height) {
  const viewport = readFloats(view, 152, 6);
  const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height
    && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL vertex-color viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12]
    .map((offset) => view.getUint32(176 + offset, true));
  if (x === 0 && y === 0 && scissorWidth === 0 && scissorHeight === 0) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) {
    throw new Error("VirGL vertex-color scissor must fit its bounded target");
  }
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function readFloats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
