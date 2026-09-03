import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay, parseGpu3dPacket } from "./gpu-display.js?v=20260903-virgl-viewport-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260903-virgl-viewport-r1";
import { virglTexturedPacket } from "./gpu-test-packets.mjs?v=20260903-virgl-viewport-r1";

test("VirGL repeat sampler envelope carries the standard sampler state", () => {
  const packet = repeatPacket({ sequence: 66 });
  const frame = parseGpu3dPacket(packet);
  assert.equal(frame.version, 5);
  assert.equal(frame.texture.addressMode, "repeat");
  const invalid = packet.slice();
  new DataView(invalid.buffer).setUint32(168, 0x1092, true);
  assert.throws(() => parseGpu3dPacket(invalid), /length or version/);
});

test("VirGL repeat sampler configures matching WebGPU address modes", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  assert.deepEqual(await display.present3d(repeatPacket({ sequence: 67 })), {
    sequence: 67, success: true,
  });
  assert.deepEqual(device.samplers, [{
    addressModeU: "repeat", addressModeV: "repeat", magFilter: "nearest",
    minFilter: "nearest", mipmapFilter: "nearest",
  }]);
});

function repeatPacket(options) {
  const source = virglTexturedPacket(options);
  const packet = new Uint8Array(source.byteLength + 4);
  packet.set(source.subarray(0, 168));
  const view = new DataView(packet.buffer);
  view.setUint32(4, 5, true);
  view.setUint32(168, 0x1080, true);
  packet.set(source.subarray(168, 176), 172);
  packet.set(source.subarray(176), 180);
  return packet;
}
