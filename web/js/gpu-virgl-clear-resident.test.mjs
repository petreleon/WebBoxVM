import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglClearPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReadbackPacket }
  from "./gpu-test-virgl-resident-readback.mjs?v=20260904-virgl-readback-pool-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL VGC1 resident clear framing names an optional predecessor", () => {
  const fresh = parseGpu3dPacket(virglClearPacket({ sequence: 75, version: 2 }));
  assert.equal(fresh.residentCandidate, true);
  assert.equal(fresh.residentPreviousProducer, undefined);
  const replacement = parseGpu3dPacket(virglClearPacket({
    residentPreviousProducer: 75, sequence: 76, version: 2,
  }));
  assert.equal(replacement.residentPreviousProducer, 75);
  assert.throws(() => parseGpu3dPacket(virglClearPacket({
    residentPreviousProducer: 76, sequence: 76, version: 2,
  })), /replacement producer/);
});

test("resident clears rekey one browser target before a later solid batch and readback", async () => {
  const pixels = new Uint8Array(1024 * 768 * 4); pixels.set([3, 2, 1, 255]);
  const device = fakeDevice({ readbackBytes: pixels });
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglClearPacket({ sequence: 75, version: 2 })), {
    resident: true, sequence: 75, success: true,
  });
  assert.equal(device.textures.length, 1);
  assert.equal(device.textureTransfers.length, 1);
  assert.deepEqual(await display.present3d(virglClearPacket({
    residentPreviousProducer: 75, sequence: 76, version: 2,
  })), { resident: true, sequence: 76, success: true });
  assert.equal(device.textures.length, 1);
  assert.equal(device.textureTransfers.length, 2);
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({
    residentPreviousProducer: 76, sequence: 77, version: 7,
  })), { resident: true, sequence: 77, success: true });
  assert.equal(device.textures.length, 1);
  assert.equal(device.textureTransfers.length, 3);
  const readback = await display.present3d(virglResidentReadbackPacket({
    producerSequence: 77, sequence: 78,
  }));
  assert.equal(readback.success, true);
  assert.deepEqual(readback.readback?.pixels.subarray(0, 4), new Uint8Array([3, 2, 1, 255]));
  assert.equal(device.textureCopies.length, 1);
  assert.equal(device.textures[0].destroyed, true);
});
