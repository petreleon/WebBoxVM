import { paddedBytesPerRow } from "./gpu-scanout-packet.js?v=20260904-virgl-solid-gpu-readback-r1";

export const READBACK_FORMAT_BGRA8 = 1;
export const READBACK_FORMAT_RGBA8 = 2;

const MAX_READBACK_BYTES = 64 * 1024 * 1024;

export function canvasConfiguration(device, format) {
  const usage = textureUsage();
  return { alphaMode: "opaque", device, format, usage: usage.COPY_SRC | usage.RENDER_ATTACHMENT };
}

export function submitTextureReadback(device, encoder, texture, width, height, format) {
  const outputFormat = readbackFormat(format);
  if (!outputFormat || typeof encoder.copyTextureToBuffer !== "function") {
    return submitWithoutReadback(device, encoder);
  }
  const bytesPerRow = paddedBytesPerRow(width);
  const bytes = checkedBytes(bytesPerRow, height);
  if (!bytes || bytes > MAX_READBACK_BYTES) return submitWithoutReadback(device, encoder);
  const buffer = device.createBuffer({
    label: "VirGL material-batch GPU readback", size: bytes,
    usage: bufferUsage().COPY_DST | bufferUsage().MAP_READ,
  });
  if (!buffer || typeof buffer.mapAsync !== "function" || typeof buffer.getMappedRange !== "function") {
    buffer?.destroy?.();
    return submitWithoutReadback(device, encoder);
  }
  encoder.copyTextureToBuffer(
    { texture }, { buffer, bytesPerRow, rowsPerImage: height },
    { depthOrArrayLayers: 1, height, width },
  );
  device.queue.submit([encoder.finish()]);
  return mappedPixels(buffer, width, height, bytesPerRow).then((pixels) => ({ format: outputFormat, pixels }));
}

async function mappedPixels(buffer, width, height, bytesPerRow) {
  try {
    await buffer.mapAsync(mapMode().READ);
    const source = new Uint8Array(buffer.getMappedRange());
    const rowBytes = width * 4;
    if (source.byteLength < bytesPerRow * height) throw new Error("WebGPU readback buffer is truncated");
    const pixels = new Uint8Array(rowBytes * height);
    for (let row = 0; row < height; row += 1) {
      pixels.set(source.subarray(row * bytesPerRow, row * bytesPerRow + rowBytes), row * rowBytes);
    }
    return pixels;
  } finally {
    buffer.unmap?.();
    buffer.destroy?.();
  }
}

function submitWithoutReadback(device, encoder) {
  device.queue.submit([encoder.finish()]);
  return Promise.resolve(device.queue.onSubmittedWorkDone()).then(() => undefined);
}

function checkedBytes(bytesPerRow, height) {
  const bytes = bytesPerRow * height;
  return Number.isSafeInteger(bytes) && bytes > 0 ? bytes : undefined;
}

function readbackFormat(format) {
  if (format === "bgra8unorm") return READBACK_FORMAT_BGRA8;
  if (format === "rgba8unorm") return READBACK_FORMAT_RGBA8;
  return undefined;
}

function bufferUsage() {
  return globalThis.GPUBufferUsage ?? { COPY_DST: 0x08, MAP_READ: 0x01 };
}

function mapMode() {
  return globalThis.GPUMapMode ?? { READ: 0x01 };
}

function textureUsage() {
  return globalThis.GPUTextureUsage ?? { COPY_SRC: 0x01, RENDER_ATTACHMENT: 0x10 };
}
