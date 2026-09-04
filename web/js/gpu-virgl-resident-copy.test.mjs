import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, extractGpu3dSequence, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentCopyPacket } from "./gpu-test-virgl-resident-copy.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReleasePacket } from "./gpu-test-virgl-resident-release.mjs?v=20260904-virgl-readback-pool-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL resident-copy packets require a bounded VRC1 envelope", () => {
  const packet = virglResidentCopyPacket({ producerSequence: 54, sequence: 55 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-resident-copy"); assert.equal(frame.offscreen, true);
  assert.equal(frame.producerSequence, 54); assert.equal(frame.residentCandidate, true);
  assert.equal(extractGpu3dSequence(packet), 55);
  new DataView(packet.buffer).setUint32(16, 0, true);
  assert.throws(() => parseGpu3dPacket(packet), /VRC1 framing/);
});

test("VirGL resident copies transfer between durable WebGPU targets without a CPU readback", async () => {
  const device = fakeDevice(); const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglSolidBatchPacket({ sequence: 75, version: 6 }));
  assert.deepEqual(await display.present3d(virglResidentCopyPacket()), { resident: true, sequence: 76, success: true });
  const outputs = device.textures.filter((texture) => texture.descriptor.label?.startsWith("VirGL resident output"));
  assert.equal(outputs.length, 2); assert.equal(device.textureCopies.length, 0);
  assert.equal(device.textureTransfers.length, 2);
  assert.equal(device.textureTransfers[1].source.texture, outputs[0]);
  assert.equal(device.textureTransfers[1].destination.texture, outputs[1]);
  assert.equal(outputs[1].descriptor.usage & 2, 2);
  await display.present3d(virglResidentReleasePacket({ producerSequence: 75 }));
  assert.equal(outputs[0].destroyed, true); assert.equal(outputs[1].destroyed, undefined);
  await display.present3d(virglResidentReleasePacket({ producerSequence: 76 }));
  assert.equal(outputs[1].destroyed, true);
});

test("VirGL resident-copy fails closed when its browser source is absent", async () => {
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(fakeDevice())]) },
  });
  assert.deepEqual(await display.present3d(virglResidentCopyPacket()), { sequence: 76, success: false });
});
