import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, extractGpu3dSequence, parseGpu3dPacket } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReadbackPacket }
  from "./gpu-test-virgl-resident-readback.mjs?v=20260904-virgl-readback-pool-r1";
import { virglResidentReleasePacket }
  from "./gpu-test-virgl-resident-release.mjs?v=20260904-virgl-readback-pool-r1";
import { virglSolidBatchPacket } from "./gpu-test-virgl-solid-batch.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL resident-readback parser retains its transfer and producer sequences", () => {
  const packet = virglResidentReadbackPacket({ producerSequence: 54, sequence: 55 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-resident-readback");
  assert.equal(frame.producerSequence, 54);
  assert.equal(extractGpu3dSequence(packet), 55);
  const invalid = packet.slice();
  new DataView(invalid.buffer).setUint32(16, 0, true);
  assert.throws(() => parseGpu3dPacket(invalid), /VGR1 framing/);
});

test("VirGL partial resident-readback parser retains its GPU source rectangle", () => {
  const packet = virglResidentReadbackPacket({
    producerSequence: 54, sequence: 55, sourceRect: { height: 5, width: 4, x: 2, y: 3 },
  });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.retainResident, true);
  assert.deepEqual(frame.readbackOrigin, { x: 2, y: 3, z: 0 });
  assert.equal(frame.readbackWidth, 4); assert.equal(frame.readbackHeight, 5);
  const invalid = virglResidentReadbackPacket({ sourceRect: { height: 1, width: 2, x: 1023, y: 0 } });
  assert.throws(() => parseGpu3dPacket(invalid), /VGR1 framing/);
});

test("VirGL resident release has no guest sequence and requires VGL1 framing", () => {
  const packet = virglResidentReleasePacket({ producerSequence: 54 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.protocol, "virgl-resident-release");
  assert.equal(frame.producerSequence, 54);
  assert.equal(extractGpu3dSequence(packet), undefined);
  new DataView(packet.buffer).setUint32(4, 2, true);
  assert.throws(() => parseGpu3dPacket(packet), /VGL1 framing/);
});

test("VirGL resident output stays on the GPU until a guest transfer asks for pixels", async () => {
  const pixels = new Uint8Array(1024 * 768 * 4); pixels.set([3, 2, 1, 255]);
  const device = fakeDevice({ readbackBytes: pixels });
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({ sequence: 75, version: 6 })), {
    resident: true, sequence: 75, success: true,
  });
  assert.equal(device.textureCopies.length, 0);
  assert.equal(device.textureTransfers.length, 1);
  const recovered = await display.present3d(virglResidentReadbackPacket({ producerSequence: 75, sequence: 76 }));
  assert.equal(recovered.sequence, 76); assert.equal(recovered.success, true);
  assert.equal(recovered.readback?.format, 1);
  assert.deepEqual(recovered.readback?.pixels.subarray(0, 4), new Uint8Array([3, 2, 1, 255]));
  assert.equal(device.textureCopies.length, 1);
  assert.equal(device.textures[0].destroyed, true);
});

test("partial resident readback maps only its source rectangle and retains the output", async () => {
  const pixels = new Uint8Array(512);
  pixels.set([3, 2, 1, 255, 7, 6, 5, 255]); pixels.set([11, 10, 9, 255, 15, 14, 13, 255], 256);
  const device = fakeDevice({ readbackBytes: pixels });
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglSolidBatchPacket({ sequence: 75, version: 6 }));
  const partial = await display.present3d(virglResidentReadbackPacket({
    producerSequence: 75, sequence: 76, sourceRect: { height: 2, width: 2, x: 1, y: 2 },
  }));
  assert.equal(partial.success, true);
  assert.deepEqual(partial.readback?.pixels, new Uint8Array([3, 2, 1, 255, 7, 6, 5, 255, 11, 10, 9, 255, 15, 14, 13, 255]));
  assert.deepEqual(device.textureCopies[0].source.origin, { x: 1, y: 2, z: 0 });
  assert.deepEqual(device.textureCopies[0].size, { depthOrArrayLayers: 1, height: 2, width: 2 });
  assert.equal(device.textures[0].destroyed, undefined);
  await display.present3d(virglResidentReadbackPacket({ producerSequence: 75, sequence: 77 }));
  assert.equal(device.textures[0].destroyed, true);
});

test("VirGL resident release drops an unread browser target without an acknowledgment", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglSolidBatchPacket({ sequence: 75, version: 6 }));
  const release = virglResidentReleasePacket();
  assert.equal(extractGpu3dSequence(release), undefined);
  assert.deepEqual(await display.present3d(release), {});
  assert.equal(device.textures[0].destroyed, true);
  assert.deepEqual(await display.present3d(virglResidentReadbackPacket()), { sequence: 76, success: false });
});

test("VirGL resident redraw rekeys one browser texture instead of allocating another", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglSolidBatchPacket({ sequence: 75, version: 6 }));
  assert.deepEqual(await display.present3d(virglSolidBatchPacket({
    residentPreviousProducer: 75, sequence: 76, version: 7,
  })), { resident: true, sequence: 76, success: true });
  assert.equal(device.textures.length, 1);
  assert.deepEqual(await display.present3d(virglResidentReleasePacket({ producerSequence: 75 })), {});
  assert.equal(device.textures[0].destroyed, undefined);
  assert.equal((await display.present3d(virglResidentReadbackPacket({
    producerSequence: 76, sequence: 77,
  }))).success, true);
  assert.equal(device.textures[0].destroyed, true);
});
