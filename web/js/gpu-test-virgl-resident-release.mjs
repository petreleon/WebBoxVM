export function virglResidentReleasePacket({ producerSequence = 75 } = {}) {
  const packet = new Uint8Array(12);
  packet.set([0x56, 0x47, 0x4c, 0x31]);
  const view = new DataView(packet.buffer);
  view.setUint32(4, 1, true);
  view.setUint32(8, producerSequence, true);
  return packet;
}
