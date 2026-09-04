import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglMaterialBatchPacket } from "./gpu-test-virgl-material-batch.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReleasePacket } from "./gpu-test-virgl-resident-release.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentSamplePacket } from "./gpu-test-virgl-resident-sample.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL resident-sample packets validate a source producer without pixel payload", () => {
  const packet = virglResidentSamplePacket(); const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 12); assert.equal(frame.residentCandidate, true); assert.equal(frame.residentSource, true);
  assert.deepEqual(frame.draws[0].texture, { addressMode: "clamp-to-edge", filter: "nearest", height: 65, producerSequence: 90, width: 65 });
  new DataView(packet.buffer).setUint32(112, 91, true);
  assert.throws(() => parseGpu3dPacket(packet), /resident texture framing/);
});

test("VirGL opaque resident samples preserve their exact replacement mask", () => {
  const frame = parseGpu3dPacket(virglResidentSamplePacket({ version: 13, writeMask: 9 }));
  assert.equal(frame.residentCandidate, true); assert.equal(frame.residentSource, true);
  assert.equal(frame.blend, "replace"); assert.equal(frame.writeMask, 9);
});

test("VirGL opaque resident samples bind their durable source without blending", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglMaterialBatchPacket({ canvasHeight: 65, canvasWidth: 65, drawCount: 1, sequence: 90, version: 2 }));
  assert.deepEqual(await display.present3d(virglResidentSamplePacket({ version: 13, writeMask: 9 })), {
    resident: true, sequence: 91, success: true,
  });
  const target = device.pipelines.at(-1).descriptor.fragment.targets[0];
  assert.equal(target.writeMask, 9); assert.equal("blend" in target, false);
  assert.equal(device.writes.length, 0); assert.equal(device.textureCopies.length, 0);
});

test("VirGL resident samples stay on WebGPU and release each durable target independently", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglMaterialBatchPacket({ canvasHeight: 65, canvasWidth: 65, drawCount: 1, sequence: 90, version: 2 })), {
    resident: true, sequence: 90, success: true,
  });
  assert.deepEqual(await display.present3d(virglResidentSamplePacket()), { resident: true, sequence: 91, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 2); assert.equal(device.writes.length, 0); assert.equal(device.textureCopies.length, 0);
  assert.equal(outputs[1].descriptor.usage & 4, 4);
  assert.equal(device.bindGroups.at(-1).entries[0].resource.texture, outputs[0]);
  await display.present3d(virglResidentReleasePacket({ producerSequence: 90 })); assert.equal(outputs[0].destroyed, true);
  await display.present3d(virglResidentReleasePacket({ producerSequence: 91 })); assert.equal(outputs[1].destroyed, true);
});

test("VirGL resident sampling fails closed when the browser lacks its producer", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglResidentSamplePacket()), { sequence: 91, success: false });
  const output = device.textures.find((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(output?.destroyed, true);
});
