const MAGIC = [0x56, 0x47, 0x43, 0x31]; // VGC1
const PACKET_BYTES = { standard: 36, resident: 40 };
const MAX_DIMENSION = 8192;

export function isVirglClearPacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function extractVirglClearSequence(packet) {
  if (!isVirglClearPacket(packet) || packet.byteLength < 12) return undefined;
  const sequence = new DataView(packet.buffer, packet.byteOffset, 12).getUint32(8, true);
  return sequence === 0 ? undefined : sequence;
}

export function parseVirglClearPacket(packet) {
  if (!(packet instanceof Uint8Array)) throw new TypeError("VirGL clear packet must be a Uint8Array");
  if (!isVirglClearPacket(packet)) throw new Error("VirGL clear packet has invalid VGC1 magic");
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  const expectedBytes = version === 1 ? PACKET_BYTES.standard : version === 2 ? PACKET_BYTES.resident : 0;
  if (packet.byteLength !== expectedBytes) throw new Error("VirGL clear packet has invalid length or version");
  const sequence = view.getUint32(8, true);
  const canvasWidth = view.getUint32(12, true);
  const canvasHeight = view.getUint32(16, true);
  if (sequence === 0) throw new Error("VirGL clear packet sequence must be nonzero");
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`VirGL clear dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  const clearColor = new Float32Array(4);
  for (let index = 0; index < clearColor.length; index += 1) {
    const value = view.getFloat32(20 + index * 4, true);
    if (!Number.isFinite(value) || value < 0 || value > 1) {
      throw new Error("VirGL clear color must contain normalized finite values");
    }
    clearColor[index] = value;
  }
  const residentPreviousProducer = version === 2 ? view.getUint32(36, true) : undefined;
  if (residentPreviousProducer === sequence) throw new Error("VirGL clear replacement producer is invalid");
  return {
    acceleration: "webgpu-virgl-capset1-clear",
    canvasHeight, canvasWidth, capsetId: 1, clearColor,
    presentationLabel: "VirGL capset 1 clear", protocol: "virgl-clear", residentCandidate: version === 2,
    residentPreviousProducer: residentPreviousProducer || undefined, sequence, version,
  };
}
