import assert from "node:assert/strict";
import test from "node:test";
import { padBgraRows, paddedBytesPerRow, parseGpuScanoutPacket }
  from "./gpu-scanout-packet.js?v=20260904-virgl-depth-r1";
import { gpuPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-depth-r1";

test("WBGF parser accepts an offset view and exposes its dirty rectangle", () => {
  const packet = gpuPacket({
    height: 1,
    pixels: [1, 2, 3, 255, 4, 5, 6, 255],
    scanoutHeight: 4,
    scanoutWidth: 5,
    width: 2,
    x: 2,
    y: 3,
  });
  const framed = new Uint8Array(packet.byteLength + 7);
  framed.set(packet, 3);
  const parsed = parseGpuScanoutPacket(framed.subarray(3, 3 + packet.byteLength));
  assert.deepEqual({ ...parsed, pixels: [...parsed.pixels] }, {
    height: 1,
    pixels: [1, 2, 3, 255, 4, 5, 6, 255],
    scanoutHeight: 4,
    scanoutWidth: 5,
    version: 1,
    width: 2,
    x: 2,
    y: 3,
  });
});

test("WBGF parser rejects bad magic, version, bounds, and payload length", () => {
  const valid = gpuPacket();
  const badMagic = valid.slice();
  badMagic[0] = 0;
  assert.throws(() => parseGpuScanoutPacket(badMagic), /invalid WBGF magic/);
  const badVersion = valid.slice();
  new DataView(badVersion.buffer).setUint32(4, 2, true);
  assert.throws(() => parseGpuScanoutPacket(badVersion), /version 2/);
  const badBounds = valid.slice();
  new DataView(badBounds.buffer).setUint32(16, 2, true);
  assert.throws(() => parseGpuScanoutPacket(badBounds), /outside the scanout/);
  assert.throws(() => parseGpuScanoutPacket(valid.subarray(0, -1)), /length mismatch/);
});

test("BGRA rows are padded to WebGPU's 256-byte row alignment", () => {
  const pixels = Uint8Array.from({ length: 24 }, (_, index) => index + 1);
  const padded = padBgraRows(pixels, 3, 2);
  assert.equal(paddedBytesPerRow(3), 256);
  assert.equal(padded.bytesPerRow, 256);
  assert.equal(padded.data.byteLength, 512);
  assert.deepEqual([...padded.data.subarray(0, 12)], [...pixels.subarray(0, 12)]);
  assert.deepEqual([...padded.data.subarray(12, 256)], new Array(244).fill(0));
  assert.deepEqual([...padded.data.subarray(256, 268)], [...pixels.subarray(12)]);
});
