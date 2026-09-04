import assert from "node:assert/strict";
import test from "node:test";
import { fakeDevice } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { VirglVertexUploadCache } from "./webgpu-virgl-vertex-cache.js?v=20260904-virgl-readback-pool-r1";

test("VirGL vertex uploads require exact bytes and the current GPU buffer", () => {
  const cache = new VirglVertexUploadCache();
  const device = fakeDevice();
  const firstBuffer = device.createBuffer({ size: 4 });
  const bytes = new Uint8Array([1, 2, 3, 4]);
  assert.equal(cache.upload(device, firstBuffer, bytes), true);
  assert.equal(cache.upload(device, firstBuffer, bytes), false);
  bytes[3] = 5;
  assert.equal(cache.upload(device, firstBuffer, bytes), true);
  const replacement = device.createBuffer({ size: 4 });
  assert.equal(cache.upload(device, replacement, bytes), true);
  cache.invalidate();
  assert.equal(cache.upload(device, replacement, bytes), true);
  assert.equal(device.bufferWrites.length, 4);
});

test("VirGL vertex upload cache does not retain oversized payloads", () => {
  const cache = new VirglVertexUploadCache();
  const device = fakeDevice();
  const buffer = device.createBuffer({ size: 4 });
  const bytes = new Uint8Array(2 * 1024 * 1024 + 1);
  cache.upload(device, buffer, bytes);
  cache.upload(device, buffer, bytes);
  assert.equal(device.bufferWrites.length, 2);
});
