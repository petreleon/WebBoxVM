export const GPU_3D_HEADER_BYTES = 48;
export const GPU_3D_OPCODE_DRAW_INDEXED = 1;
export const GPU_3D_VERSION = 1;

import {
  extractVirglClearSequence,
  isVirglClearPacket,
  parseVirglClearPacket,
} from "./virgl-clear-packet.js?v=20260904-virgl-depth-compare-r1";
import {
  extractVirglDrawSequence,
  isVirglDrawPacket,
  parseVirglDrawPacket,
} from "./virgl-draw-packet.js?v=20260904-virgl-depth-compare-r1";
import {
  extractVirglSolidBatchSequence,
  isVirglSolidBatchPacket,
  parseVirglSolidBatchPacket,
} from "./virgl-solid-batch-packet.js?v=20260904-virgl-depth-compare-r1";

const MAGIC = [0x57, 0x42, 0x47, 0x33]; // WBG3
const MAX_DIMENSION = 8192;
const MAX_VERTICES = 4096;
const MAX_INDICES = 12288;
const MVP_FLOATS = 16;
const VERTEX_FLOATS = 7;
const FIXED_BYTES = GPU_3D_HEADER_BYTES + MVP_FLOATS * 4;

export function extractGpu3dSequence(packet) {
  if (isVirglSolidBatchPacket(packet)) return extractVirglSolidBatchSequence(packet);
  if (isVirglClearPacket(packet)) return extractVirglClearSequence(packet);
  if (isVirglDrawPacket(packet)) return extractVirglDrawSequence(packet);
  if (!(packet instanceof Uint8Array) || packet.byteLength < 16) return undefined;
  for (let index = 0; index < MAGIC.length; index += 1) {
    if (packet[index] !== MAGIC[index]) return undefined;
  }
  const sequence = new DataView(packet.buffer, packet.byteOffset, 16).getUint32(12, true);
  return sequence === 0 ? undefined : sequence;
}

export function parseGpu3dPacket(packet) {
  if (isVirglSolidBatchPacket(packet)) return parseVirglSolidBatchPacket(packet);
  if (isVirglClearPacket(packet)) return parseVirglClearPacket(packet);
  if (isVirglDrawPacket(packet)) return parseVirglDrawPacket(packet);
  if (!(packet instanceof Uint8Array)) throw new TypeError("GPU 3D packet must be a Uint8Array");
  if (packet.byteLength < FIXED_BYTES) {
    throw new Error("GPU 3D packet is shorter than its header and MVP matrix");
  }
  for (let index = 0; index < MAGIC.length; index += 1) {
    if (packet[index] !== MAGIC[index]) throw new Error("GPU 3D packet has invalid WBG3 magic");
  }
  const view = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const version = view.getUint32(4, true);
  const opcode = view.getUint32(8, true);
  const sequence = view.getUint32(12, true);
  const canvasWidth = view.getUint32(16, true);
  const canvasHeight = view.getUint32(20, true);
  const vertexCount = view.getUint32(24, true);
  const indexCount = view.getUint32(28, true);
  if (version !== GPU_3D_VERSION) throw new Error(`Unsupported GPU 3D packet version ${version}`);
  if (opcode !== GPU_3D_OPCODE_DRAW_INDEXED) throw new Error(`Unsupported GPU 3D opcode ${opcode}`);
  if (!canvasWidth || !canvasHeight || canvasWidth > MAX_DIMENSION || canvasHeight > MAX_DIMENSION) {
    throw new Error(`GPU 3D canvas dimensions must be between 1 and ${MAX_DIMENSION}`);
  }
  if (vertexCount > MAX_VERTICES) throw new Error(`GPU 3D vertex count exceeds ${MAX_VERTICES}`);
  if (indexCount > MAX_INDICES) throw new Error(`GPU 3D index count exceeds ${MAX_INDICES}`);
  if (indexCount % 3 !== 0) throw new Error("GPU 3D index count must be divisible by 3");
  const expected = FIXED_BYTES + vertexCount * VERTEX_FLOATS * 4 + indexCount * 2;
  if (packet.byteLength !== expected) {
    throw new Error(`GPU 3D packet length mismatch: expected ${expected}, got ${packet.byteLength}`);
  }
  const clearColor = readFloats(view, 32, 4, "clear color");
  const mvp = readFloats(view, GPU_3D_HEADER_BYTES, MVP_FLOATS, "MVP");
  const vertexOffset = FIXED_BYTES;
  const vertices = readFloats(view, vertexOffset, vertexCount * VERTEX_FLOATS, "vertex");
  const indexOffset = vertexOffset + vertices.byteLength;
  const indices = new Uint16Array(indexCount);
  for (let index = 0; index < indexCount; index += 1) {
    const value = view.getUint16(indexOffset + index * 2, true);
    if (value >= vertexCount) {
      throw new Error(`GPU 3D index ${value} is outside ${vertexCount} vertices`);
    }
    indices[index] = value;
  }
  return {
    acceleration: "webgpu-experimental-capset",
    canvasHeight,
    canvasWidth,
    capsetId: 7,
    clearColor,
    indexCount,
    indices,
    mvp,
    opcode,
    presentationLabel: "experimental guest 3D",
    protocol: "wbg3",
    sequence,
    vertexCount,
    vertices,
    version,
  };
}

function readFloats(view, offset, count, label) {
  const values = new Float32Array(count);
  for (let index = 0; index < count; index += 1) {
    const value = view.getFloat32(offset + index * 4, true);
    if (!Number.isFinite(value)) throw new Error(`GPU 3D ${label} contains a non-finite value`);
    values[index] = value;
  }
  return values;
}
