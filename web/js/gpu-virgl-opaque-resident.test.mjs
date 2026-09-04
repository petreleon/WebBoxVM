import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMaterialBatchPacket } from "./gpu-test-virgl-material-batch.mjs?v=20260904-virgl-readback-pool-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-readback-pool-r1";

test("opaque solid batches retain and rekey their GPU target without a readback", async () => {
  const fresh = virglSolidBatchPacket({ drawCount: 1, sequence: 106, version: 14, writeMask: 15 });
  const replacement = virglSolidBatchPacket({
    drawCount: 1, residentPreviousProducer: 106, sequence: 107, version: 15, writeMask: 15,
  });
  const frame = parseGpu3dPacket(fresh); const rekey = parseGpu3dPacket(replacement);
  assert.equal(frame.blend, "replace"); assert.equal(frame.residentCandidate, true); assert.equal(frame.writeMask, 15);
  assert.equal(rekey.residentPreviousProducer, 106); assert.equal(rekey.writeMask, 15);
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(fresh), { resident: true, sequence: 106, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 1); assert.equal(device.textureCopies.length, 0); assert.equal(device.textureTransfers.length, 1);
  assert.deepEqual(await display.present3d(replacement), { resident: true, sequence: 107, success: true });
  assert.equal(outputs.length, 1); assert.equal(device.textureCopies.length, 0); assert.equal(device.textureTransfers.length, 2);
  assert.deepEqual(device.pipelines[0].descriptor.fragment.targets, [{ format: "bgra8unorm" }]);
});

test("opaque material batches retain masked targets and preserve their replacement producer", async () => {
  const fresh = virglMaterialBatchPacket({ drawCount: 1, sequence: 108, version: 10, writeMask: 9 });
  const replacement = virglMaterialBatchPacket({
    drawCount: 1, residentPreviousProducer: 108, sequence: 109, version: 11, writeMask: 9,
  });
  const frame = parseGpu3dPacket(fresh); const rekey = parseGpu3dPacket(replacement);
  assert.equal(frame.blend, "replace"); assert.equal(frame.residentCandidate, true); assert.equal(frame.writeMask, 9);
  assert.equal(rekey.residentPreviousProducer, 108); assert.equal(rekey.writeMask, 9);
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(fresh), { resident: true, sequence: 108, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 1); assert.equal(device.textureCopies.length, 0); assert.equal(device.textureTransfers.length, 1);
  assert.deepEqual(await display.present3d(replacement), { resident: true, sequence: 109, success: true });
  assert.equal(outputs.length, 1); assert.equal(device.textureCopies.length, 0); assert.equal(device.textureTransfers.length, 2);
  const target = device.pipelines[0].descriptor.fragment.targets[0];
  assert.equal("blend" in target, false); assert.equal(target.writeMask, 9); assert.equal(target.format, "bgra8unorm");
});
