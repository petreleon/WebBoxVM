import assert from "node:assert/strict";
import test from "node:test";
import { GuestDisplay } from "./gpu-display.js?v=20260904-virgl-readback-pool-r1";
import { bindRunnerEvents } from "./runner-events.js?v=20260904-virgl-readback-pool-r1";
import { fakeCanvas, fakeStatus } from "./gpu-test-fakes.mjs?v=20260904-virgl-readback-pool-r1";
import { gpu3dPacket } from "./gpu-test-packets.mjs?v=20260904-virgl-readback-pool-r1";

test("3D acknowledgment waits for completion and is suppressed after VM replacement", async () => {
  let current = true;
  let finishFirst;
  let finishStale;
  const acknowledgments = [];
  const completions = [
    new Promise((resolve) => { finishFirst = resolve; }),
    new Promise((resolve) => { finishStale = resolve; }),
  ];
  const emulator = {
    gpu3d_ack: (...values) => acknowledgments.push(values),
  };
  bindRunnerEvents(emulator, {
    autosave() {},
    current: () => current,
    error() {},
    frame2d() {},
    frame3d: () => completions.shift(),
    metrics() {},
    network() {},
    uart() {},
  });

  emulator.onGpu3dFrame(new Uint8Array([1]));
  await Promise.resolve();
  assert.deepEqual(acknowledgments, []);
  finishFirst({ sequence: 10, success: true });
  await Promise.resolve();
  await Promise.resolve();
  assert.deepEqual(acknowledgments, [[10, true]]);

  emulator.onGpu3dFrame(new Uint8Array([2]));
  current = false;
  finishStale({ sequence: 11, success: true });
  await Promise.resolve();
  await Promise.resolve();
  assert.deepEqual(acknowledgments, [[10, true]]);
});

test("malformed WBG3 geometry is negatively acknowledged by device sequence", async () => {
  const acknowledgments = [];
  const display = new GuestDisplay(fakeCanvas(), fakeStatus(), { navigator: {} });
  const emulator = { gpu3d_ack: (...values) => acknowledgments.push(values) };
  bindRunnerEvents(emulator, {
    autosave() {}, current: () => true, error() {}, frame2d() {},
    frame3d: (packet) => display.present3d(packet), metrics() {}, network() {}, uart() {},
  });
  const packet = gpu3dPacket({ sequence: 88 });
  new DataView(packet.buffer).setUint16(packet.byteLength - 2, 99, true);
  emulator.onGpu3dFrame(packet);
  await Promise.resolve();
  await Promise.resolve();
  assert.deepEqual(acknowledgments, [[88, false]]);
});

test("GPU readback follows the matching 3D acknowledgment", async () => {
  const acknowledgments = []; const pixels = new Uint8Array([1, 2, 3, 4]);
  const emulator = { gpu3d_ack: (...values) => acknowledgments.push(values) };
  bindRunnerEvents(emulator, {
    autosave() {}, current: () => true, error() {}, frame2d() {}, metrics() {}, network() {}, uart() {},
    frame3d: () => ({ readback: { format: 1, pixels }, sequence: 9, success: true }),
  });
  emulator.onGpu3dFrame(new Uint8Array([1])); await Promise.resolve();
  assert.deepEqual(acknowledgments, [[9, true, { format: 1, pixels }]]);
});
