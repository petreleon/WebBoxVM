const MATRIX = [
  0.5, 0, 0, 0.25,
  0, 0.5, 0, 0,
  0, 0, 1, 0,
  0, 0, 0, 1,
];

export function virglMatrixPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  drawColor = [0, 1, 0, 0.25],
  matrix = MATRIX,
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 7,
  vertices = triangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 4; const state = 120 + vertices.length * 4;
  const packet = new Uint8Array(state + 40); const view = new DataView(packet.buffer);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  [15, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  floats(view, 24, clearColor); floats(view, 40, drawColor); floats(view, 56, matrix); floats(view, 120, vertices); floats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  return packet;
}

function floats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}

function triangle() {
  return [0, 0.75, 0, 1, -0.75, -0.75, 0, 1, 0.75, -0.75, 0, 1];
}
