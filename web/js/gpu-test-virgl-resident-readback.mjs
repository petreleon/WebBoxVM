export function virglResidentReadbackPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  producerSequence = 75,
  sequence = 76,
  sourceRect,
} = {}) {
  const partial = sourceRect !== undefined;
  const packet = new Uint8Array(partial ? 40 : 24);
  packet.set([0x56, 0x47, 0x52, 0x31]);
  const view = new DataView(packet.buffer);
  [partial ? 2 : 1, sequence, producerSequence, canvasWidth, canvasHeight]
    .forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  if (partial) [sourceRect.x, sourceRect.y, sourceRect.width, sourceRect.height]
    .forEach((value, index) => view.setUint32(24 + index * 4, value, true));
  return packet;
}
