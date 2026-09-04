import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { virglTexturedPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-readback-pool-r1";

test("VirGL textured draws upload only byte-changed sampler pixels", async () => {
  const device = fakeDevice();
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  await display.present3d(virglTexturedPacket({ sequence: 45 }));
  await display.present3d(virglTexturedPacket({ sequence: 46 }));
  assert.equal(device.writes.length, 1);
  await display.present3d(virglTexturedPacket({
    sequence: 47, texture: [11, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255],
  }));
  assert.equal(device.writes.length, 2);
});
