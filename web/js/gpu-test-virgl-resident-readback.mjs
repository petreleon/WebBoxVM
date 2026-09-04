export function virglResidentReadbackPacket({
  canvasHeight = 768,
  canvasWidth = 1024,
  producerSequence = 75,
  sequence = 76,
} = {}) {
  const packet = new Uint8Array(24);
  packet.set([0x56, 0x47, 0x52, 0x31]);
  const view = new DataView(packet.buffer);
  [1, sequence, producerSequence, canvasWidth, canvasHeight]
    .forEach((value, index) => view.setUint32(4 + index * 4, value, true));
  return packet;
}
