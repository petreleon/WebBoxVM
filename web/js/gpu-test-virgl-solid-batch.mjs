export function virglSolidBatchPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  clearColor = [0, 0, 0, 1],
  version = 1,
  resident = [6, 7].includes(version),
  depthCompare = version === 2 ? 1 : 0,
  depthWriteEnabled = true,
  depthClear = [1, 6, 7, 8, 10, 12].includes(version) ? 0 : 1,
  drawCount = 2,
  draws = defaultDraws(canvasWidth, canvasHeight).slice(0, drawCount),
  residentPreviousProducer = version === 7 ? 72 : undefined,
  sequence = 73,
  writeMask = 9,
} = {}) {
  const stateBytes = [4, 5, 9, 11, 13].includes(version) ? 64 : 60;
  const body = draws.reduce((total, draw) => total + stateBytes + draw.vertices.length * 4, 0);
  const headerBytes = version === 7 ? 52 : 48;
  const packet = new Uint8Array(headerBytes + body);
  packet.set([0x56, 0x47, 0x42, 0x31]);
  const view = new DataView(packet.buffer);
  [version, sequence, canvasWidth, canvasHeight, draws.length, version === 3 ? depthCompare : [6, 7].includes(version) && resident ? 1 : [12, 13].includes(version) ? writeMask : 0]
    .forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  writeFloats(view, 28, clearColor);
  view.setFloat32(44, depthClear, true);
  if (version === 7) view.setUint32(48, residentPreviousProducer, true);
  let offset = headerBytes;
  for (const draw of draws) {
    const vertexCount = draw.vertices.length / 4;
    view.setUint32(offset, vertexCount, true);
    const state = offset + ([4, 5, 9, 11, 13].includes(version) ? 4 : 0);
    if (version === 4) view.setUint32(offset + 4, draw.depthCompare ?? depthCompare, true);
    if ([5, 9, 11, 13].includes(version)) {
      const compare = draw.depthCompare ?? depthCompare;
      const write = draw.depthWriteEnabled ?? depthWriteEnabled;
      view.setUint32(offset + 4, 1 | (write ? 2 : 0) | (compare << 2), true);
    }
    writeFloats(view, state + 4, draw.drawColor);
    writeFloats(view, state + 20, draw.viewport);
    draw.scissor.forEach((value, index) => view.setUint32(state + 44 + index * 4, value, true));
    writeFloats(view, offset + stateBytes, draw.vertices);
    offset += stateBytes + draw.vertices.length * 4;
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
