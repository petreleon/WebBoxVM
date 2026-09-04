const MAGIC = [0x56, 0x47, 0x52, 0x31];
const FULL_PACKET_BYTES = 24;
const PARTIAL_PACKET_BYTES = 40;
const MAX_DIMENSION = 8192;

export function isVirglResidentReadbackPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglResidentReadbackSequence(packet) {
  if (!isVirglResidentReadbackPacket(packet) || packet.byteLength < 12) return undefined;
  return new DataView(packet.buffer, packet.byteOffset, packet.byteLength).getUint32(8, true) || undefined;
}

export function parseVirglResidentReadbackPacket(packet) {
  if (!isVirglResidentReadbackPacket(packet) || packet.byteLength < 8) {
    throw new Error("VirGL resident-readback packet has invalid VGR1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const packetVersion = view.getUint32(4, true);
  const packetBytes = packetVersion === 1 ? FULL_PACKET_BYTES : packetVersion === 2 ? PARTIAL_PACKET_BYTES : 0;
  if (packet.byteLength !== packetBytes) throw new Error("VirGL resident-readback packet has invalid VGR1 framing");
  const [version, sequence, producerSequence, canvasWidth, canvasHeight] = [4, 8, 12, 16, 20]
    .map((offset) => view.getUint32(offset, true));
  if (!sequence || !producerSequence || !canvasWidth || !canvasHeight
    || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error("VirGL resident-readback packet has invalid VGR1 framing");
  }
  const [x, y, width, height] = version === 2 ? [24, 28, 32, 36]
    .map((offset) => view.getUint32(offset, true)) : [0, 0, canvasWidth, canvasHeight];
  if (!width || !height || width > canvasWidth || height > canvasHeight
    || x > canvasWidth - width || y > canvasHeight - height) {
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
    ...(version === 2 && {
      readbackHeight: height,
      readbackOrigin: { x, y, z: 0 },
      readbackWidth: width,
      retainResident: true,
    }),
    sequence,
    version,
  };
}
