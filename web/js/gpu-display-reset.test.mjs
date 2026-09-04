import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-depth-texture-color-r1";
import { fakeAdapter, fakeCanvas, fakeDevice, fakeGpu, fakeStatus }
  from "./gpu-test-fakes.mjs?v=20260904-virgl-depth-texture-color-r1";
import { gpu3dPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-depth-texture-color-r1";

function displayFor(device, status = fakeStatus()) {
  const display = new GuestDisplay(fakeCanvas({ webgpu: true }), status, {
    navigator: { gpu: fakeGpu([fakeAdapter(device)]) },
  });
  return { display, status };
}

function observeFirstSubmit(device) {
  let resolve;
  const submitted = new Promise((done) => { resolve = done; });
  const submit = device.queue.submit;
  device.queue.submit = (commands) => {
    submit(commands);
    if (device.submits === 1) resolve();
  };
  return submitted;
}

test("reset clears a completed 3D-only canvas through the shared session", async () => {
  const device = fakeDevice();
  const { display } = displayFor(device);
  assert.equal((await display.present3d(gpu3dPacket())).success, true);
  assert.equal(device.submits, 1);
  display.reset();
  assert.equal(device.submits, 2);
  assert.deepEqual(device.renderPasses.at(-1).colorAttachments[0], {
    clearValue: { a: 1, b: 0, g: 0, r: 0 },
    loadOp: "clear",
    storeOp: "store",
    view: { kind: "canvas-view" },
  });
});

test("reset clear is queued after an already submitted WBG3 draw", async () => {
  let finishWork;
  const workDone = new Promise((resolve) => { finishWork = resolve; });
  const device = fakeDevice({ workDone });
  const submitted = observeFirstSubmit(device);
  const { display } = displayFor(device);
  const draw = display.present3d(gpu3dPacket());
  await submitted;
  display.reset();
  assert.equal(device.submits, 2);
  finishWork();
  assert.deepEqual(await draw, { sequence: 7, success: false });
});

test("destroy queues the shared clear before destroying the device", async () => {
  const events = [];
  const device = fakeDevice();
  const submit = device.queue.submit;
  device.queue.submit = (commands) => { events.push("submit"); submit(commands); };
  device.destroy = () => events.push("destroy");
  const { display } = displayFor(device);
  await display.present3d(gpu3dPacket());
  events.length = 0;
  display.destroy();
  assert.deepEqual(events, ["submit", "destroy"]);
});

test("a rejected old WBG3 promise cannot overwrite reset diagnostics", async () => {
  let rejectWork;
  const workDone = new Promise((_, reject) => { rejectWork = reject; });
  const device = fakeDevice({ workDone });
  const submitted = observeFirstSubmit(device);
  const { display, status } = displayFor(device);
  const draw = display.present3d(gpu3dPacket());
  await submitted;
  display.reset();
  rejectWork(new Error("stale queue rejection"));
  assert.deepEqual(await draw, { sequence: 7, success: false });
  assert.equal(status.dataset.threeDAcceleration, "inactive");
  assert.equal(status.dataset.threeDErrors, "0");
  assert.equal(status.dataset.threeDLastError, "");
  assert.equal(status.textContent, "Waiting for guest display (webgpu ready)");
});

test("a rejected WBG3 promise cannot overwrite device-loss diagnostics", async () => {
  let rejectWork;
  const workDone = new Promise((_, reject) => { rejectWork = reject; });
  const device = fakeDevice({ workDone });
  const submitted = observeFirstSubmit(device);
  const { display, status } = displayFor(device);
  const draw = display.present3d(gpu3dPacket());
  await submitted;
  device.lose({ message: "expected test loss" });
  await Promise.resolve();
  rejectWork(new Error("stale loss rejection"));
  assert.deepEqual(await draw, { sequence: 7, success: false });
  assert.equal(status.dataset.backend, "recovering");
  assert.equal(status.dataset.threeDErrors, "0");
  assert.equal(status.dataset.threeDLastError, "");
});
