import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260930-resident-lifetime-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260930-resident-lifetime-r1";
import { virglClearPacket } from "./gpu-test-packets.mjs?v=20260930-resident-lifetime-r1";
import { virglResidentReadbackPacket }
  from "./gpu-test-virgl-resident-readback.mjs?v=20260930-resident-lifetime-r1";
import { virglResidentReleasePacket }
  from "./gpu-test-virgl-resident-release.mjs?v=20260930-resident-lifetime-r1";

function displayFor(device) {
  return new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
}

function deferred() {
  let resolve;
  const promise = new Promise((done) => { resolve = done; });
  return { promise, resolve };
}

function observeSubmit(device) {
  const submitted = deferred();
  const submit = device.queue.submit;
  device.queue.submit = (commands) => { submit(commands); submitted.resolve(); };
  return submitted.promise;
}

test("release waits for an accepted resident producer to finish publication", async () => {
  const work = deferred(); const device = fakeDevice({ workDone: work.promise });
  const submitted = observeSubmit(device); const display = displayFor(device);
  const draw = display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  await submitted;
  const release = display.present3d(virglResidentReleasePacket({ producerSequence: 75 }));
  assert.equal(device.textures[0].destroyed, undefined);
  work.resolve();
  assert.deepEqual(await draw, { resident: true, sequence: 75, success: true });
  assert.deepEqual(await release, {});
  await display.whenIdle();
  assert.equal(device.textures[0].destroyed, true);
  assert.deepEqual(await display.present3d(virglResidentReadbackPacket()), { sequence: 76, success: false });
  display.destroy();
});

test("release preserves an earlier queued readback's pixels", async () => {
  const pixels = new Uint8Array(1024 * 768 * 4); pixels.set([31, 47, 63, 255]);
  const device = fakeDevice({ readbackBytes: pixels }); const display = displayFor(device);
  await display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  const readback = display.present3d(virglResidentReadbackPacket());
  const release = display.present3d(virglResidentReleasePacket());
  const result = await readback;
  assert.equal(result.sequence, 76); assert.equal(result.success, true);
  assert.deepEqual(result.readback.pixels.subarray(0, 4), new Uint8Array([31, 47, 63, 255]));
  assert.deepEqual(await release, {}); assert.equal(device.textures[0].destroyed, true);
  display.destroy();
});

test("a release queued before reset cannot destroy a new producer with the same sequence", async () => {
  const work = deferred(); const device = fakeDevice({ workDone: work.promise });
  const submitted = observeSubmit(device); const display = displayFor(device);
  const old = display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  await submitted;
  const release = display.present3d(virglResidentReleasePacket());
  display.reset();
  const fresh = display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  work.resolve();
  assert.deepEqual(await old, { sequence: 75, success: false });
  assert.deepEqual(await release, {});
  assert.deepEqual(await fresh, { resident: true, sequence: 75, success: true });
  assert.equal(device.textures[0].destroyed, true); assert.equal(device.textures[1].destroyed, undefined);
  display.destroy(); assert.equal(device.textures[1].destroyed, true);
});

test("device loss cancels old producer/control work before the new generation reuses its sequence", async () => {
  const work = deferred(); const first = fakeDevice({ workDone: work.promise });
  const second = fakeDevice(); const submitted = observeSubmit(first);
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), fakeStatus(), {
    navigator: { gpu: fakeGpu([fakeAdapter(first), fakeAdapter(second)]) },
  });
  const old = display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  await submitted;
  const release = display.present3d(virglResidentReleasePacket());
  first.lose({ message: "generation replacement" }); await Promise.resolve();
  const fresh = display.present3d(virglClearPacket({ sequence: 75, version: 2 }));
  work.resolve();
  assert.deepEqual(await old, { sequence: 75, success: false });
  assert.deepEqual(await release, {});
  assert.deepEqual(await fresh, { resident: true, sequence: 75, success: true });
  assert.equal(first.textures[0].destroyed, true); assert.equal(second.textures[0].destroyed, undefined);
  display.destroy(); assert.equal(second.textures[0].destroyed, true);
});
