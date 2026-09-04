const MAGIC = [0x56, 0x47, 0x4c, 0x31];
const PACKET_BYTES = 12;

export function isVirglResidentReleasePacket(packet) {
  return packet instanceof Uint8Array && MAGIC.every((byte, index) => packet[index] === byte);
}

export function parseVirglResidentReleasePacket(packet) {
  if (!isVirglResidentReleasePacket(packet) || packet.byteLength !== PACKET_BYTES) {
    throw new Error("VirGL resident-release packet has invalid VGL1 framing");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  const producerSequence = view.getUint32(8, true);
  if (version !== 1 || !producerSequence) {
    throw new Error("VirGL resident-release packet has invalid VGL1 framing");
  }
  return { control: "resident-release", producerSequence, protocol: "virgl-resident-release", version };
}
