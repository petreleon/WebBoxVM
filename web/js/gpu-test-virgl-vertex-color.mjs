export function virglVertexColorPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  depthState,
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 7,
  vertices = colorTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const vertexCount = vertices.length / 8;
  const state = 56 + vertices.length * 4;
  const packet = new Uint8Array(state + (depthState ? 48 : 40));
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [depthState ? 12 : 7, sequence, canvasWidth, canvasHeight, vertexCount].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor);
  writeFloats(view, 56, vertices);
  writeFloats(view, state, viewport);
  scissor.forEach((value, index) => view.setUint32(state + 24 + index * 4, value, true));
  if (depthState) {
    view.setFloat32(state + 40, depthState.clear ?? 1, true);
    view.setUint32(state + 44, 1 | (depthState.write !== false ? 2 : 0) | ((depthState.compare ?? 1) << 2), true);
  }
  return packet;
}

function colorTriangle() {
  return [
    0, 0.75, 0, 1, 1, 0, 0, 1,
    -0.75, -0.75, 0, 1, 0, 1, 0, 1,
    0.75, -0.75, 0, 1, 0, 0, 1, 1,
  ];
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}
