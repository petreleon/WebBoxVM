const MAX_DIMENSION = 8192;
const MAX_VERTEX_COUNT = 3063;
const MATRIX_OFFSET = 56;
const VERTEX_OFFSET = MATRIX_OFFSET + 64;
const STATE_BYTES = 40;

export function parseVirglMatrixPacket(packet) {
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  if (packet.byteLength < VERTEX_OFFSET || view.getUint32(4, true) !== 15) throw new Error("VirGL matrix packet has invalid version or length");
  const sequence = view.getUint32(8, true); const canvasWidth = view.getUint32(12, true);
  const canvasHeight = view.getUint32(16, true); const vertexCount = view.getUint32(20, true);
  if (!sequence) throw new Error("VirGL matrix packet sequence must be nonzero");
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) throw new Error("VirGL matrix dimensions must fit the bounded target");
  if (vertexCount < 3 || vertexCount > MAX_VERTEX_COUNT || vertexCount % 3) throw new Error("VirGL matrix vertex count must be 3..3063 and divisible by 3");
  const state = VERTEX_OFFSET + vertexCount * 16;
  if (packet.byteLength !== state + STATE_BYTES) throw new Error("VirGL matrix packet has invalid length");
  const clearColor = color(view, 24, "clear"); const drawColor = color(view, 40, "draw");
  const matrix = floats(view, MATRIX_OFFSET, 16);
  if (![...matrix].every(Number.isFinite)) throw new Error("VirGL matrix rows must be finite");
  const vertices = floats(view, VERTEX_OFFSET, vertexCount * 4);
  if (!validProjected(vertices, matrix)) throw new Error("VirGL matrix projection is invalid");
  return {
    acceleration: "webgpu-virgl-capset1-matrix", canvasHeight, canvasWidth, capsetId: 1,
    clearColor, drawColor, matrix, presentationLabel: "VirGL capset 1 GPU matrix triangles",
    protocol: "virgl-draw", sequence, version: 15, vertexCount, vertices,
    ...viewportState(view, canvasWidth, canvasHeight, state),
  };
}

function validProjected(vertices, matrix) {
  const projected = new Float32Array(vertices.length);
  for (let base = 0; base < vertices.length; base += 4) {
    const input = vertices.subarray(base, base + 4);
    const output = [0, 1, 2, 3].map((row) => dot(matrix.subarray(row * 4, row * 4 + 4), input));
    const w = output[3]; const normal = output.slice(0, 3).map((value) => Math.fround(value / w));
    if (!Number.isFinite(w) || w <= 0 || !normal.every((value) => Number.isFinite(value) && value >= -1 && value <= 1)) return false;
    projected.set([...normal, 1], base);
  }
  return validPositions(projected);
}

function dot(row, input) {
  let total = 0;
  for (let index = 0; index < 4; index += 1) total = Math.fround(total + Math.fround(row[index] * input[index]));
  return total;
}

function validPositions(vertices) {
  for (let base = 0; base < vertices.length; base += 12) {
    const points = [0, 1, 2].map((index) => vertices.subarray(base + index * 4, base + index * 4 + 4));
    if (!points.every((point) => point[3] === 1 && [...point.subarray(0, 3)].every((value) => Number.isFinite(value) && value >= -1 && value <= 1))) return false;
    if (Math.abs(edge(points[0], points[1], points[2])) < 0.001) return false;
  }
  return true;
}

function edge(a, b, point) {
  const left = Math.fround(Math.fround(point[0] - a[0]) * Math.fround(b[1] - a[1]));
  const right = Math.fround(Math.fround(point[1] - a[1]) * Math.fround(b[0] - a[0]));
  return Math.fround(left - right);
}

function color(view, offset, label) {
  const values = floats(view, offset, 4);
  if (![...values].every((value) => Number.isFinite(value) && value >= 0 && value <= 1)) throw new Error(`VirGL matrix ${label} color is invalid`);
  return values;
}

function viewportState(view, width, height, state) {
  const viewport = floats(view, state, 6); const [sx, sy, sz, tx, ty, tz] = viewport;
  const valid = viewport.every(Number.isFinite) && sx > 0 && sy > 0 && sz >= 0
    && tx - sx >= 0 && tx + sx <= width && ty - sy >= 0 && ty + sy <= height && tz - sz >= 0 && tz + sz <= 1;
  if (!valid) throw new Error("VirGL matrix viewport must fit its bounded target");
  const [x, y, scissorWidth, scissorHeight] = [0, 4, 8, 12].map((offset) => view.getUint32(state + 24 + offset, true));
  if (!scissorWidth && !scissorHeight && !x && !y) return { viewport };
  if (!scissorWidth || !scissorHeight || x + scissorWidth > width || y + scissorHeight > height) throw new Error("VirGL matrix scissor must fit its bounded target");
  return { viewport, scissor: { x, y, width: scissorWidth, height: scissorHeight } };
}

function floats(view, offset, count) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) values[index] = view.getFloat32(offset + index * 4, true);
  return values;
}
