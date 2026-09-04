import assert from "node:assert/strict";
import test from "node:test";
import { fakeDevice } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { VirglTextureSnapshotCache } from "./webgpu-virgl-texture-cache.js?v=20260904-virgl-readback-pool-r1";

test("VirGL texture snapshot cache requires byte-identical content", () => {
  const cache = new VirglTextureSnapshotCache(); const device = fakeDevice(); const retired = [];
  const first = { addressMode: "clamp-to-edge", filter: "nearest", height: 1, pixels: new Uint8Array([1, 2, 3, 4]), width: 1 };
  const changed = { ...first, pixels: new Uint8Array([1, 2, 3, 5]) };
  cache.bindGroupEntries(device, [first], retired); cache.bindGroupEntries(device, [first], retired);
  cache.bindGroupEntries(device, [changed], retired);
  assert.equal(device.writes.length, 2); assert.equal(retired.length, 0);
});

test("VirGL texture bindings reuse only exact cached snapshots", () => {
  const cache = new VirglTextureSnapshotCache(); const device = fakeDevice(); const retired = [];
  const pipeline = device.createRenderPipeline({});
  const first = { addressMode: "clamp-to-edge", filter: "nearest", height: 1, pixels: new Uint8Array([1, 2, 3, 4]), width: 1 };
  const changed = { ...first, pixels: new Uint8Array([1, 2, 3, 5]) };
  const group = cache.bindGroup(device, pipeline, [first], retired);
  assert.equal(cache.bindGroup(device, pipeline, [first], retired), group);
  assert.notEqual(cache.bindGroup(device, pipeline, [changed], retired), group);
  assert.equal(device.bindGroups.length, 2);
});

test("VirGL texture eviction drops bind groups that could name retired textures", () => {
  const cache = new VirglTextureSnapshotCache(); const device = fakeDevice(); const retired = [];
  const pipeline = device.createRenderPipeline({});
  const snapshot = (value) => ({ addressMode: "clamp-to-edge", filter: "nearest", height: 1, pixels: new Uint8Array([value, 2, 3, 4]), width: 1 });
  const first = snapshot(0); const group = cache.bindGroup(device, pipeline, [first], retired);
  for (let value = 1; value <= 32; value += 1) cache.bindGroup(device, pipeline, [snapshot(value)], retired);
  assert.notEqual(cache.bindGroup(device, pipeline, [first], retired), group);
  assert.ok(retired.length >= 1);
});
