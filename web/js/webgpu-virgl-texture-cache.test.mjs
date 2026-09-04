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
