export function virglSolidBatchPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0, 0, 0, 1],
  version = 1,
  depthClear = version === 2 ? 1 : 0,
  draws = defaultDraws(canvasWidth, canvasHeight),
  sequence = 73,
} = {}) {
  const body = draws.reduce((total, draw) => total + 60 + draw.vertices.length * 4, 0);
  const packet = new Uint8Array(48 + body);
  packet.set([0x56, 0x47, 0x42, 0x31]);
  const view = new DataView(packet.buffer);
  [version, sequence, canvasWidth, canvasHeight, draws.length, 0]
    .forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 28, clearColor);
  view.setFloat32(44, depthClear, true);
  let offset = 48;
  for (const draw of draws) {
    const vertexCount = draw.vertices.length / 4;
    view.setUint32(offset, vertexCount, true);
    writeFloats(view, offset + 4, draw.drawColor);
    writeFloats(view, offset + 20, draw.viewport);
    draw.scissor.forEach((value, index) => view.setUint32(offset + 44 + index * 4, value, true));
    writeFloats(view, offset + 60, draw.vertices);
    offset += 60 + draw.vertices.length * 4;
  }
  return packet;
}

function defaultDraws(width, height) {
  const viewport = [width / 2, height / 2, 0.5, width / 2, height / 2, 0.5];
  const vertices = [0, 0.75, 0, 1, -0.75, -0.75, 0, 1, 0.75, -0.75, 0, 1];
  return [
    { drawColor: [1, 0, 0, 0.5], scissor: [0, 0, width, height], vertices, viewport },
    { drawColor: [0, 1, 0, 0.5], scissor: [0, 0, width, height], vertices, viewport },
  ];
}

function writeFloats(view, offset, values) {
  values.forEach((value, index) => view.setFloat32(offset + index * 4, value, true));
}
