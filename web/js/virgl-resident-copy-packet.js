const MAGIC = [0x56, 0x52, 0x43, 0x31];
const PACKET_BYTES = 24;
const MAX_DIMENSION = 8192;

export function isVirglResidentCopyPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglResidentCopySequence(packet) {
  if (!isVirglResidentCopyPacket(packet) || packet.byteLength < 12) return undefined;
  return new DataView(packet.buffer, packet.byteOffset, packet.byteLength).getUint32(8, true) || undefined;
}

export function parseVirglResidentCopyPacket(packet) {
  if (!isVirglResidentCopyPacket(packet) || packet.byteLength !== PACKET_BYTES) {
    throw new Error("VirGL resident-copy packet has invalid VRC1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, producerSequence, canvasWidth, canvasHeight] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  if (version !== 1 || !sequence || !producerSequence || sequence === producerSequence
    || !canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error("VirGL resident-copy packet has invalid VRC1 framing");
  }
  return {
    acceleration: "webgpu-virgl-capset1-resident-copy",
    canvasHeight, canvasWidth, capsetId: 1, offscreen: true,
    presentationLabel: "VirGL resident GPU texture copy", producerSequence,
    protocol: "virgl-resident-copy", residentCandidate: true, sequence, version,
  };
}
