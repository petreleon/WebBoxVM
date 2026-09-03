export const GPU_SCANOUT_HEADER_BYTES = 32;
export const GPU_SCANOUT_VERSION = 1;
export const WEBGPU_BYTES_PER_ROW_ALIGNMENT = 256;

const MAGIC = [0x57, 0x42, 0x47, 0x46]; // WBGF
const MAX_SCANOUT_BYTES = 256 * 1024 * 1024;

export function parseGpuScanoutPacket(packet) {
  if (!(packet instanceof Uint8Array)) {
    throw new TypeError("GPU scanout packet must be a Uint8Array");
  }
  if (packet.byteLength < GPU_SCANOUT_HEADER_BYTES) {
    throw new Error("GPU scanout packet is shorter than its 32-byte header");
  }
  for (let index = 0; index < MAGIC.length; index += 1) {
    if (packet[index] !== MAGIC[index]) {
      throw new Error("GPU scanout packet has invalid WBGF magic");
    }
  }

  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  if (version !== GPU_SCANOUT_VERSION) {
    throw new Error(`Unsupported GPU scanout packet version ${version}`);
  }
  const scanoutWidth = view.getUint32(8, true);
  const scanoutHeight = view.getUint32(12, true);
  const x = view.getUint32(16, true);
  const y = view.getUint32(20, true);
  const width = view.getUint32(24, true);
  const height = view.getUint32(28, true);
  if (!scanoutWidth || !scanoutHeight || !width || !height) {
    throw new Error("GPU scanout dimensions and dirty rectangle must be non-zero");
  }
  if (x + width > scanoutWidth || y + height > scanoutHeight) {
    throw new Error("GPU scanout dirty rectangle is outside the scanout");
  }
  const scanoutBytes = BigInt(scanoutWidth) * BigInt(scanoutHeight) * 4n;
  if (scanoutBytes > BigInt(MAX_SCANOUT_BYTES)) {
    throw new Error("GPU scanout exceeds the 256 MiB display safety limit");
  }
  const expected = BigInt(GPU_SCANOUT_HEADER_BYTES) + BigInt(width) * BigInt(height) * 4n;
  if (expected !== BigInt(packet.byteLength)) {
    throw new Error(`GPU scanout payload length mismatch: expected ${expected}, got ${packet.byteLength}`);
  }
  return {
    height,
    pixels: packet.subarray(GPU_SCANOUT_HEADER_BYTES),
    scanoutHeight,
    scanoutWidth,
    version,
    width,
    x,
    y,
  };
}

export function paddedBytesPerRow(width) {
  if (!Number.isSafeInteger(width) || width <= 0) {
    throw new RangeError("Row width must be a positive integer");
  }
  const bytes = width * 4;
  return Math.ceil(bytes / WEBGPU_BYTES_PER_ROW_ALIGNMENT) * WEBGPU_BYTES_PER_ROW_ALIGNMENT;
}

export function padBgraRows(pixels, width, height) {
  if (!(pixels instanceof Uint8Array)) {
    throw new TypeError("BGRA pixels must be a Uint8Array");
  }
  const rowBytes = width * 4;
  if (!Number.isSafeInteger(height) || height <= 0 || pixels.byteLength !== rowBytes * height) {
    throw new RangeError("BGRA pixel length does not match the rectangle dimensions");
  }
  const bytesPerRow = paddedBytesPerRow(width);
  if (bytesPerRow === rowBytes) return { bytesPerRow, data: pixels };
  const data = new Uint8Array(bytesPerRow * height);
  for (let row = 0; row < height; row += 1) {
    data.set(pixels.subarray(row * rowBytes, (row + 1) * rowBytes), row * bytesPerRow);
  }
  return { bytesPerRow, data };
}
