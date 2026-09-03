export function virglVertexColorPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0.1, 0.2, 0.3, 1],
  scissor = [0, 0, canvasWidth, canvasHeight],
  sequence = 7,
  vertices = colorTriangle(),
  viewport = [canvasWidth / 2, canvasHeight / 2, 0.5, canvasWidth / 2, canvasHeight / 2, 0.5],
} = {}) {
  const packet = new Uint8Array(192);
  packet.set([0x56, 0x47, 0x44, 0x31]);
  const view = new DataView(packet.buffer);
  [7, sequence, canvasWidth, canvasHeight, 3].forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 24, clearColor);
  writeFloats(view, 56, vertices);
  writeFloats(view, 152, viewport);
  scissor.forEach((value, index) => view.setUint32(176 + index * 4, value, true));
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
