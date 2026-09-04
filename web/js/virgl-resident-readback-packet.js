const MAGIC = [0x56, 0x47, 0x52, 0x31];
const PACKET_BYTES = 24;
const MAX_DIMENSION = 8192;

export function isVirglResidentReadbackPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglResidentReadbackSequence(packet) {
  if (!isVirglResidentReadbackPacket(packet) || packet.byteLength < 12) return undefined;
  return new DataView(packet.buffer, packet.byteOffset, packet.byteLength).getUint32(8, true) || undefined;
}

export function parseVirglResidentReadbackPacket(packet) {
  if (!isVirglResidentReadbackPacket(packet) || packet.byteLength !== PACKET_BYTES) {
    throw new Error("VirGL resident-readback packet has invalid VGR1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const [version, sequence, producerSequence, canvasWidth, canvasHeight] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  if (version !== 1 || !sequence || !producerSequence || !canvasWidth || !canvasHeight
    || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error("VirGL resident-readback packet has invalid VGR1 framing");
  }
  return {
    acceleration: "webgpu-virgl-capset1-resident-readback",
    canvasHeight,
    canvasWidth,
    capsetId: 1,
    presentationLabel: "VirGL resident texture readback",
    producerSequence,
    protocol: "virgl-resident-readback",
    sequence,
    version,
  };
}
