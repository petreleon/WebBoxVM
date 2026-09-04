export function virglDepthPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  depthClear = 1,
  drawColor = [0, 1, 0, 0.25],
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 70,
  vertices = triangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 4;
  const state = 56 + vertices.length * 4;
  const packet = new Uint8Array(state + 44);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [9, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor); writeFloats(view, 40, drawColor); writeFloats(view, 56, vertices);
  writeFloats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  view.setFloat32(state + 40, depthClear, true);
  return packet;
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}

function triangle() {
  return [0, 0.75, -0.5, 1, -0.75, -0.75, -0.5, 1, 0.75, -0.75, -0.5, 1];
}
